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

pub fn get_routes() -> Vec<Route> {
    routes![
        authorize,
        authorize_form,
        index,
        login,
        logout,
        single_use_token,
        register,
        register_form,
    ]
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> Template {
    let data = cookies.get(COOKIE_NAME);
    match data {
        None => render_login(""),
        Some(data) => Template::render(
            "account",
            context! {
                name: data.value().to_string()
            },
        ),
    }
}

fn render_login(redirect: &str) -> Template {
    // todo generate csrf
    Template::render(
        "login",
        context! {
            redirect: redirect
        },
    )
}

#[derive(FromForm, Debug)]
struct Login<'r> {
    email: &'r str,
    password: &'r str,
    redirect: &'r str,
}

const COOKIE_NAME: &str = "RSESSID";

#[post("/login", data = "<login>")]
async fn login(
    login: Form<Login<'_>>,
    state_repository: &State<AuthUserRepository>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    let key = get_model_key(login.email);

    let model = state_repository.get_model(&key).await.map_err(|e| {
        dbg!(e);
        Status::InternalServerError
    })?;

    let user = model.state();

    let Some(password_hash) = user.password_hash() else {
        dbg!("no password");
        return Err(Status::InternalServerError);
    };

    let parsed_hash = PasswordHash::new(&password_hash).map_err(|e| {
        dbg!(e);
        Status::InternalServerError
    })?;
    assert!(Argon2::default()
        .verify_password(login.password.as_ref(), &parsed_hash)
        .is_ok());

    state_repository
        .add_command(&key, AuthUserCommand::Login, None)
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?;

    let mut cookie = Cookie::new(COOKIE_NAME, key.format());
    cookie.set_same_site(Some(SameSite::Lax));

    cookies.add(cookie);

    dbg!(&login.redirect);

    if !login.redirect.is_empty() {
        let redirect = login.redirect.to_string();
        dbg!(&redirect);
        return Ok(Redirect::temporary(uri!(authorize(redirect))));
    }

    Ok(Redirect::to(uri!(index)))
}

#[get("/register")]
async fn register(cookies: &CookieJar<'_>) -> Template {
    let _data = cookies.get(COOKIE_NAME);
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

fn get_model_key(email: &str) -> ModelKey {
    ModelKey::new(
        AUTH_USER_STREAM,
        Uuid::new_v5(&Uuid::NAMESPACE_X500, email.as_ref()),
    )
}

#[post("/register", data = "<register>")]
async fn register_form(
    state_repository: &State<AuthUserRepository>,
    register: Form<Register<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    let _data = cookies.get(COOKIE_NAME);

    let key = get_model_key(register.email);

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

    Ok(Redirect::to(uri!(index)))
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(COOKIE_NAME);

    Redirect::to(uri!(index))
}

#[get("/authorize?<redirect>")]
async fn authorize(
    cookies: &CookieJar<'_>,
    maria_db: &State<MariadDb>,
    redirect: &str,
) -> Result<Template, Status> {
    dbg!(&redirect);

    let application = maria_db
        .get_application_by_host(redirect)
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?
        .ok_or(Status::NotFound)?;

    let data = cookies.get(COOKIE_NAME);
    match data {
        None => Ok(render_login(redirect)),
        Some(_) => Ok(Template::render(
            "authorize",
            context! {
                application_name: application.name(),
                redirect: redirect
            },
        )),
    }
}

#[derive(FromForm, Debug)]
struct Authorize<'r> {
    redirect: &'r str,
}

#[post("/authorize", data = "<authorize>")]
async fn authorize_form(
    cookies: &CookieJar<'_>,
    maria_db: &State<MariadDb>,
    authorize: Form<Authorize<'_>>,
) -> Result<Redirect, Status> {
    let application = maria_db
        .get_application_by_host(authorize.redirect)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let data = cookies.get(COOKIE_NAME).ok_or(Status::NotFound)?;

    let account = maria_db
        .get_user_by_id(data.value())
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let id = maria_db
        .new_one_time_token(&application, &account)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to(format!(
        "{}/auth?token={id}",
        authorize.redirect
    )))
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
