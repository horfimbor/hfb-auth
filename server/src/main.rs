mod controllers;

#[macro_use]
extern crate rocket;

use crate::controllers::helper::hash_password;
use anyhow::Context;
use clap::Parser;
use eventstore::Client;
use hfb_auth_shared::user::{AuthUserCommand, AuthUserState, UserRole};
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::cache_db::redis::StateDb;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::{Repository, StateRepository};
use rocket::fs::{relative, FileServer};
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket_dyn_templates::{context, Template};
use std::env;
use std::path::Path;
use url::quirks::password;
use uuid::Uuid;
use hfb_auth_shared::application::AuthApplicationState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    real_env: bool,

    #[arg(short, long)]
    user: Option<Uuid>,

    #[arg(long)]
    role: Option<UserRole>,

    #[arg(long)]
    password: Option<String>,
}

type AuthUserStateCache = StateDb<AuthUserState>;
type AuthUserRepository = StateRepository<AuthUserState, AuthUserStateCache>;

type ApplicationStateCache = StateDb<AuthApplicationState>;
type ApplicationRepository = StateRepository<AuthApplicationState, ApplicationStateCache>;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    if !args.real_env {
        dotenvy::dotenv().context("cannot get env")?;
    }

    let eventstore_uri = env::var("EVENTSTORE_URI")
        .context("fail to get EVENTSTORE_URI env var")?
        .parse()
        .context("fail to parse the settings")?;

    let redis_client =
        redis::Client::open(env::var("REDIS_URI").context("fail to get REDIS_URI env var")?)?;

    let event_store_db = Client::new(eventstore_uri).context("fail to connect to eventstore db")?;

    let auth_user_repository = AuthUserRepository::new(
        event_store_db.clone(),
        AuthUserStateCache::new(redis_client.clone()),
    );

    if let Some(user) = args.user {
        let key = ModelKey::new(AUTH_USER_STREAM, user);

        if let Some(role) = args.role {
            let admin = auth_user_repository
                .add_command(&key, AuthUserCommand::ChangeRole(Some(role)), None)
                .await
                .context("cannot change role")?;
            dbg!(&admin);
        }
        if let Some(password) = args.password {
            let admin = auth_user_repository
                .add_command(
                    &key,
                    AuthUserCommand::ChangePassword {
                        password_hash: hash_password(&password).unwrap(),
                    },
                    None,
                )
                .await
                .context("cannot reset password")?;
            dbg!(&admin);
        }
        return Ok(());
    }

    let auth_port = env::var("APP_PORT")
        .context("APP_PORT is not defined")?
        .parse::<u16>()
        .context("APP_PORT cannot be parse in u16")?;
    let auth_host = env::var("APP_HOST").context("APP_HOST is not defined")?;

    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", auth_port))
        .merge(("template_dir", "server/templates"));

    let allowed_origins = AllowedOrigins::some_exact(&[auth_host]);

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .context("cannot create cors")?;

    let _ = rocket::custom(figment)
        .manage(auth_user_repository)
        .mount("/", controllers::get_routes())
        .mount("/", FileServer::from(relative!("web")))
        .attach(cors)
        .attach(Template::fairing())
        .register("/", catchers![general_not_found, internal_error])
        .launch()
        .await;

    Ok(())
}

#[catch(404)]
fn general_not_found() -> Template {
    Template::render("404", context! {})
}

#[catch(500)]
fn internal_error() -> Template {
    Template::render("500", context! {})
}
