use crate::controllers::COOKIE_SESSION;
use crate::{controllers, AuthUserRepository};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use hfb_auth_shared::user::AuthUserCommand;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

#[get("/register")]
pub async fn register(cookies: &CookieJar<'_>) -> Template {
    let _data = cookies.get(COOKIE_SESSION);
    Template::render(
        "register",
        context! {
            redirect: ""
        },
    )
}

#[derive(FromForm, Debug)]
struct Register<'r> {
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
    let _data = cookies.get(COOKIE_SESSION);

    let key = controllers::get_model_key(register.email);

    if register.password != register.password_check {
        todo!("handle password diff");
    }
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(register.password.as_ref(), &salt)
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?
        .to_string();

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

    Ok(Redirect::to(uri!(controllers::index)))
}
