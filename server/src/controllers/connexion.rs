use crate::controllers::url_parsing::RedirectUrl;
use crate::controllers::{AUTH_USER_UUID, COOKIE_ERROR, COOKIE_SESSION};
use crate::{controllers, AuthUserRepository};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hfb_auth_shared::user::{AuthUserCommand, AuthUserState};
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::Repository;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use url::Url;

pub fn render_login(redirect: Option<Url>, error: Option<&str>) -> Template {
    // todo generate csrf
    Template::render(
        "login",
        context! {
            redirect: redirect,
            error: error
        },
    )
}

#[derive(FromForm, Debug)]
pub struct Login<'r> {
    email: &'r str,
    password: &'r str,
    redirect: &'r str,
}

#[post("/login", data = "<login>")]
pub async fn login(
    login: Form<Login<'_>>,
    auth_user_repository: &State<AuthUserRepository>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Redirect> {
    let key = ModelKey::new_uuid_v8(AUTH_USER_STREAM, AUTH_USER_UUID, login.email);

    let model = auth_user_repository
        .get_model(&key)
        .await
        .map_err(|event_source_error| {
            dbg!(event_source_error);
            redirect_failed_login(&login, cookies, "Database error")
        })?;

    let user = model.state();

    if *user == AuthUserState::default() {
        return Err(redirect_failed_login(&login, cookies, "Login failed"));
    }

    let Some(password_hash) = user.password_hash() else {
        return Err(redirect_failed_login(
            &login,
            cookies,
            "Database password corrupted",
        ));
    };

    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|_| redirect_failed_login(&login, cookies, "Password hashing failed"))?;

    assert!(Argon2::default()
        .verify_password(login.password.as_ref(), &parsed_hash)
        .is_ok());

    auth_user_repository
        .add_command(&key, AuthUserCommand::Login, None)
        .await
        .map_err(|e| {
            dbg!(e);
            redirect_failed_login(&login, cookies, "Failed to save login")
        })?;

    dbg!(&login.redirect);

    let mut cookie = Cookie::new(COOKIE_SESSION, key.format());
    cookie.set_same_site(Some(SameSite::Lax));
    cookies.add(cookie);

    if !login.redirect.is_empty() {
        let redirect = login.redirect.to_string();
        dbg!(&redirect);
        return Ok(Redirect::temporary(uri!(
            controllers::authorization::authorize(redirect)
        )));
    }

    Ok(Redirect::to(uri!(controllers::index)))
}

fn redirect_failed_login(login: &Form<Login>, cookies: &CookieJar, error: &str) -> Redirect {
    let mut cookie = Cookie::new(COOKIE_ERROR, format!("{}|{}", error, login.redirect));
    cookie.set_same_site(Some(SameSite::Lax));

    cookies.add(cookie);

    Redirect::to(uri!(controllers::index))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(COOKIE_SESSION);

    Redirect::to(uri!(controllers::index))
}
