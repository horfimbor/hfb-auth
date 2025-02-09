use redis::Client as RedisClient;
use anyhow::{Context, Error};
use std::env;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket::http::Method;
use rocket::fs::FileServer;
use rocket::___internal_relative as relative;
use rocket_dyn_templates::{context, Template};
use crate::{user, AccountRepository, ApplicationRepository, UserRepository};

pub mod admin;
pub mod authorization;
pub mod public;

pub async fn start_server(
    auth_user_repository: UserRepository,
    application_repository: ApplicationRepository,
    account_repository: AccountRepository,
    redis: RedisClient,
) -> Result<(), Error> {
    let auth_port = env::var("APP_PORT")
        .context("APP_PORT is not defined")?
        .parse::<u16>()
        .context("APP_PORT cannot be parse in u16")?;
    let auth_host = env::var("APP_HOST").context("APP_HOST is not defined")?;

    let cookie_secret_key =
        env::var("COOKIE_SECRET_KEY").context("COOKIE_SECRET_KEY must be provided")?;

    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", auth_port))
        .merge(("template_dir", "server/templates"))
        .merge(("secret_key", cookie_secret_key));

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
        .manage(application_repository)
        .manage(account_repository)
        .manage(redis)
        .mount("/", admin::get_admin_routes())
        .mount("/", authorization::get_authorization_routes())
        .mount("/", user::get_user_routes())
        .mount("/", public::get_routes())
        .mount("/", FileServer::from(relative!("web")))
        .attach(cors)
        .attach(Template::fairing())
        .register("/", catchers![general_not_found, internal_error])
        .launch()
        .await
        .context("rocket failed");

    Ok(())
}

#[catch(404)]
pub fn general_not_found() -> Template {
    Template::render("404", context! {})
}

#[catch(500)]
pub fn internal_error() -> Template {
    Template::render("500", context! {})
}