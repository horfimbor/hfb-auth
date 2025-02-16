use crate::constants::AUTH_USER_UUID;
use crate::session::{get_session, remove_session, set_session, LoggedInUser};
use crate::url_parsing::RedirectUrl;
use crate::web::error::ErrorPage;
use crate::web::{admin, application, public};
use crate::{other_error_page, user, UserRepository};
use anyhow::anyhow;
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
pub async fn index(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let mut session = get_session(cookies).map_err(|e| other_error_page!(e))?;

    Ok(render_login(None, None))
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
    identity: &'r str,
    password: &'r str,
}

#[post("/login", data = "<login>")]
pub async fn login(
    login: Form<Login<'_>>,
    auth_user_repository: &State<UserRepository>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let key = ModelKey::new_uuid_v8(AUTH_USER_STREAM, AUTH_USER_UUID, login.identity);

    let model = auth_user_repository
        .get_model(&key)
        .await
        .map_err(|e| other_error_page!(e))?;

    let user = model.state();

    if *user == UserState::default() {
        return Err(other_error_page!("Login failed"));
    }

    let Some(password_hash) = user.password_hash() else {
        return Err(other_error_page!("Database password corrupted"));
    };

    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| other_error_page!(e))?;

    assert!(Argon2::default()
        .verify_password(login.password.as_ref(), &parsed_hash)
        .is_ok());

    let user = auth_user_repository
        .add_command(&key, UserCommand::Login, None)
        .await
        .map_err(|e| other_error_page!(e))?;

    let mut session = get_session(cookies).map_err(|e| other_error_page!(e))?;
    session.set_user(Some(LoggedInUser {
        user_id: key,
        pseudo: user.pseudo().to_string(),
        is_admin: user.is_admin(),
    }));

    set_session(cookies, session.clone());

    Ok(Redirect::to(format!("{}", session.redirect_url())))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    remove_session(cookies);

    Redirect::to(uri!(public::index))
}
