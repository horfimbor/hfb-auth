use crate::constants::AUTH_USER_UUID;
use crate::public::helper;
use crate::{public, AuthUserRepository};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use hfb_auth_shared::user::AuthUserCommand;
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
    email: &'r str,
    pseudo: &'r str,
    password: &'r str,
    password_check: &'r str,
    redirect: &'r str,
}

#[post("/register", data = "<register>")]
pub async fn register_form(
    state_repository: &State<AuthUserRepository>,
    register: Form<Register<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    let key = ModelKey::new_uuid_v8(AUTH_USER_STREAM, AUTH_USER_UUID, register.email);

    if register.password != register.password_check {
        todo!("handle password diff");
    }
    let password_hash = helper::hash_password(register.password)?;

    state_repository
        .add_command(
            &key,
            AuthUserCommand::Create {
                pseudo: register.pseudo.to_string(),
                password_hash,
            },
            None,
        )
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?;

    Ok(Redirect::to(uri!(public::index)))
}
