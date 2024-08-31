use crate::model::MariadDb;
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_routes() -> Vec<Route> {
    routes![
        index,
        login,
        logout,
        authorize,
        authorize_form,
        single_use_token
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
        "index",
        context! {
            redirect: redirect
        },
    )
}

#[derive(FromForm, Debug)]
struct Login<'r> {
    pseudo: &'r str,
    redirect: &'r str,
}

const COOKIE_NAME: &str = "RSESSID";

#[post("/login", data = "<login>")]
async fn login(
    login: Form<Login<'_>>,
    maria_db: &State<MariadDb>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    let user = maria_db.get_user(login.pseudo).await.map_err(|e| {
        dbg!(e);
        Status::InternalServerError
    })?;

    let user = match user {
        Some(u) => u,
        None => maria_db.create_user(login.pseudo).await.map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?,
    };

    let mut cookie = Cookie::new(COOKIE_NAME, user.uuid().to_string());
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
