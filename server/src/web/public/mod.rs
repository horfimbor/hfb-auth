pub mod connexion;
pub mod helper;
mod registration;

use crate::anyhow;
use crate::session::get_session;
use crate::web::error::ErrorPage;
use crate::{other_error_page, user};
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use uuid::Uuid;

pub fn get_routes() -> Vec<Route> {
    routes![
        index,
        connexion::index,
        connexion::login,
        connexion::logout,
        registration::register,
        registration::register_form,
    ]
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    let session = get_session(cookies).map_err(|e| other_error_page!(e))?;

    Ok(match session.user() {
        None => Redirect::to(uri!(connexion::index())),
        Some(data) => Redirect::to(uri!(user::index())),
    })
}
