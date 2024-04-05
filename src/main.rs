mod controllers;
mod model;

#[macro_use]
extern crate rocket;

use clap::Parser;
use model::MariadDb;
use rocket::fs::{relative, FileServer};
use rocket::http::Method;
use rocket::response::content;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket_dyn_templates::Template;
use sqlx::migrate::Migrator;
use sqlx::MySqlPool;
use std::env;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    real_env: bool,
}

#[rocket::main]
async fn main() {
    let args = Args::parse();

    if !args.real_env {
        dotenvy::dotenv().expect("cannot get env");
    }

    let mariadb_url = env::var("MARIADB_URL").expect("MARIADB_URL is not defined");
    let auth_port = env::var("AUTH_PORT")
        .expect("AUTH_PORT is not defined")
        .parse::<u16>()
        .expect("AUTH_PORT cannot be parse in u16");
    let auth_host = env::var("AUTH_HOST").expect("AUTH_HOST is not defined");

    let pool = MySqlPool::connect_lazy(&mariadb_url).unwrap();

    let m = Migrator::new(Path::new("./migrations")).await.unwrap();

    m.run(&pool).await.unwrap();

    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", auth_port))
        .merge(("template_dir", "templates"));

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
    .unwrap();

    let _ = rocket::custom(figment)
        .manage(MariadDb::new(pool))
        .mount("/", controllers::get_routes())
        .mount("/", FileServer::from(relative!("web")))
        .attach(cors)
        .attach(Template::fairing())
        .register("/", catchers![general_not_found])
        .launch()
        .await;
}

#[catch(404)]
fn general_not_found() -> content::RawHtml<&'static str> {
    content::RawHtml(
        r#"
        <p>Hmm... This is not the droïd you are looking for</p>
    "#,
    )
}
