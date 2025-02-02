use crate::constants::AUTH_USER_UUID;
use crate::session::{get_session, remove_session, set_session, LoggedInUser, SessionError};
use crate::url_parsing::RedirectUrl;
use crate::{admin, authorization, public, user, AuthUserRepository};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hfb_auth_shared::user::{UserCommand, UserState};
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::Repository;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use url::Url;

#[get("/login")]
pub async fn index(cookies: &CookieJar<'_>) -> Template {
    let mut error = None;
    let mut session = get_session(cookies);
    if let Some(e) = session.error() {
        error = Some(e.clone());
        session.set_error(None);
        set_session(cookies, session.clone());
    }

    match error {
        None => render_login(None, None),
        Some(error) => render_login(
            Some(session.redirect_url().clone()),
            Some(error.message.as_str()),
        ),
    }
}

fn render_login(redirect: Option<Url>, error: Option<&str>) -> Template {
    // todo generate csrf
    Template::render(
        "login",
        context! {
            error: error
        },
    )
}

#[derive(FromForm, Debug)]
pub struct Login<'r> {
    email: &'r str,
    password: &'r str,
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

    if *user == UserState::default() {
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

    let user = auth_user_repository
        .add_command(&key, UserCommand::Login, None)
        .await
        .map_err(|e| {
            dbg!(e);
            redirect_failed_login(&login, cookies, "Failed to save login")
        })?;

    let mut session = get_session(cookies);
    session.set_user(Some(LoggedInUser {
        user_id: key,
        pseudo: user.pseudo().to_string(),
        is_admin: user.is_admin(),
    }));

    set_session(cookies, session.clone());

    Ok(Redirect::to(format!("{}", session.redirect_url())))
}

fn redirect_failed_login(login: &Form<Login>, cookies: &CookieJar, error: &str) -> Redirect {
    let mut cookie = get_session(cookies);

    cookie.set_error(Some(SessionError {
        message: error.to_string(),
    }));

    set_session(cookies, cookie);

    Redirect::to(uri!(public::index))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    remove_session(cookies);

    Redirect::to(uri!(public::index))
}
