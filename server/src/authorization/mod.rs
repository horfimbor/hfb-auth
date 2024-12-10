use crate::public::connexion;
use crate::url_parsing::RedirectUrl;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use crate::user::User;

pub fn get_authorization_routes() -> Vec<Route> {
    routes![authorize, authorize_form, single_use_token,]
}



#[get("/auth/authorize?<redirect>", rank=2)]
pub async fn authorize_guest(user: User, redirect: RedirectUrl) -> Redirect {
    todo!();
}

#[get("/auth/authorize?<redirect>")]
pub async fn authorize(user: User, redirect: RedirectUrl) -> Result<Template, Status> {
    dbg!(&redirect);

    todo!();
    //
    // let application = maria_db
    //     .get_application_by_host(redirect.host())
    //     .await
    //     .map_err(|e| {
    //         dbg!(e);
    //         Status::InternalServerError
    //     })?
    //     .ok_or(Status::NotFound)?;
    //
    // let data = cookies.get(COOKIE_SESSION);
    // match data {
    //     None => Ok(connexion::render_login(Some(redirect.url()), None)),
    //     Some(_) => Ok(Template::render(
    //         "authorize",
    //         context! {
    //             application_name: application.name(),
    //             redirect: redirect.url().as_str()
    //         },
    //     )),
    // }
}

#[derive(FromForm, Debug)]
pub struct Authorize {
    redirect: RedirectUrl,
}

#[post("/auth/authorize", data = "<authorize>")]
pub async fn authorize_form(
    cookies: &CookieJar<'_>,
    authorize: Form<Authorize>,
) -> Result<Redirect, Status> {
    todo!();
    // let application = maria_db
    //     .get_application_by_host(authorize.clone().redirect.host)
    //     .await
    //     .map_err(|_| Status::InternalServerError)?
    //     .ok_or(Status::NotFound)?;
    //
    // let data = cookies.get(COOKIE_SESSION).ok_or(Status::NotFound)?;
    //
    // let account = maria_db
    //     .get_user_by_id(data.value())
    //     .await
    //     .map_err(|_| Status::InternalServerError)?
    //     .ok_or(Status::NotFound)?;
    //
    // let id = maria_db
    //     .new_one_time_token(&application, &account)
    //     .await
    //     .map_err(|_| Status::InternalServerError)?;
    //
    // Ok(Redirect::to(format!(
    //     "{}/auth?token={id}",
    //     authorize.redirect.url
    // )))
}

#[derive(FromForm, Debug)]
pub struct SingleUseToken<'r> {
    token: &'r str,
    app_key: &'r str,
}

#[post("/auth/single-use-token", data = "<token>")]
pub async fn single_use_token(token: Form<SingleUseToken<'_>>) -> Result<String, Status> {
    todo!();
    //
    // let token = maria_db
    //     .get_one_time_token(token.token)
    //     .await
    //     .map_err(|_| Status::InternalServerError)?
    //     .ok_or(Status::NotFound)?;
    //
    // dbg!(&token);
    //
    // let application = maria_db
    //     .get_application(token.application_id())
    //     .await
    //     .map_err(|_| Status::InternalServerError)?
    //     .ok_or(Status::NotFound)?;
    //
    // dbg!(&application);
    //
    // let start = SystemTime::now();
    // let since_the_epoch = start
    //     .duration_since(UNIX_EPOCH)
    //     .map_err(|_| Status::InternalServerError)?
    //     .as_secs();
    //
    // let claims = Claims {
    //     aud: application
    //         .name()
    //         .parse()
    //         .map_err(|_| Status::InternalServerError)?,
    //     exp: (since_the_epoch + 3600) as usize,
    //     iat: since_the_epoch as usize,
    //     iss: "login".parse().unwrap(),
    //     sub: "user".parse().unwrap(),
    //     id: token.account_id().parse().unwrap(),
    // };
    //
    // let token = encode(
    //     &Header::default(),
    //     &claims,
    //     &EncodingKey::from_secret(application.app_key().as_ref()),
    // )
    // .map_err(|_| Status::InternalServerError)?;
    //
    // Ok(token)
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
