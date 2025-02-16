use crate::constants::AUTH_USER_UUID;
use crate::web::error::ErrorPage;
use crate::web::public;
use crate::web::public::helper;
use crate::{anyhow_error_page, other_error_page, UserRepository};
use anyhow::anyhow;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use hfb_auth_shared::user::UserCommand;
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

#[get("/register")]
pub async fn register(cookies: &CookieJar<'_>) -> Template {
    Template::render("register", context! {})
}

#[derive(FromForm, Debug)]
pub struct Register<'r> {
    identity: &'r str,
    pseudo: &'r str,
    password: &'r str,
    password_check: &'r str,
}

#[post("/register", data = "<register>")]
pub async fn register_form(
    state_repository: &State<UserRepository>,
    register: Form<Register<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let key = ModelKey::new_uuid_v8(AUTH_USER_STREAM, AUTH_USER_UUID, register.identity);

    if register.password != register.password_check {
        return Err(other_error_page!(
            "password is not the same as password check"
        ));
    }
    let password_hash =
        helper::hash_password(register.password).map_err(|e| anyhow_error_page!(e))?;

    state_repository
        .add_command(
            &key,
            UserCommand::Create {
                pseudo: register.pseudo.to_string(),
                password_hash,
            },
            None,
        )
        .await
        .map_err(|e| other_error_page!(e))?;

    Ok(Redirect::to(uri!(public::index)))
}
