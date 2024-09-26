mod connexion;
mod registration;
mod application;
mod authorization;

use crate::model::MariadDb;
use crate::AuthUserRepository;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hfb_auth_shared::user::AuthUserCommand;
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::Repository;
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use log::error;

pub fn get_routes() -> Vec<Route> {
    routes![
        authorization::authorize,
        authorization::authorize_form,
        single_use_token,
        index,
        connexion::login,
        connexion::logout,
        registration::register,
        registration::register_form,
    ]
}
const COOKIE_SESSION: &str = "RUSTSESSID";
const COOKIE_ERROR: &str = "SPACE_X";


#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> Template {
    let session = cookies.get(COOKIE_SESSION);
    match session {
        None => {
            let error: Option<&str> = cookies.get(COOKIE_ERROR).map(|v| v.value());
            cookies.remove(COOKIE_ERROR);
            match error {
                None => {
                    connexion::render_login("", None)}
                Some(str) => {
                    let mut s = str.split('|');
                    let error = s.next();
                    let redirect = s.next();
                    connexion::render_login(redirect.unwrap_or_default(), error)
                }
            }
        },
        Some(data) => Template::render(
            "account",
            context! {
                name: data.value().to_string()
            },
        ),
    }
}


fn get_model_key(email: &str) -> ModelKey {
    ModelKey::new(
        AUTH_USER_STREAM,
        Uuid::new_v5(&Uuid::NAMESPACE_X500, email.as_ref()),
    )
}

#[derive(FromForm, Debug)]
struct SingleUseToken<'r> {
    token: &'r str,
    app_key: &'r str,
}

#[post("/single-use-token", data = "<token>")]
async fn single_use_token(
    maria_db: &State<MariadDb>,
    token: Form<SingleUseToken<'_>>,
) -> Result<String, Status> {
    let token = maria_db
        .get_one_time_token(token.token)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    dbg!(&token);

    let application = maria_db
        .get_application(token.application_id())
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    dbg!(&application);

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Status::InternalServerError)?
        .as_secs();

    let claims = Claims {
        aud: application
            .name()
            .parse()
            .map_err(|_| Status::InternalServerError)?,
        exp: (since_the_epoch + 3600) as usize,
        iat: since_the_epoch as usize,
        iss: "login".parse().unwrap(),
        sub: "user".parse().unwrap(),
        id: token.account_id().parse().unwrap(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(application.app_key().as_ref()),
    )
    .map_err(|_| Status::InternalServerError)?;

    Ok(token)
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    // nbf: usize,          // Optional. Not Before (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
    id: String,
}
