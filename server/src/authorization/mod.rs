use crate::constants::AUTH_APPLICATION_UUID;
use crate::public::connexion;
use crate::session::{base_host, get_session, set_session};
use crate::url_parsing::RedirectUrl;
use crate::user::User;
use crate::{ApplicationRepository, AuthUserRepository};
use hfb_auth_shared::application::AuthApplicationState;
use hfb_auth_shared::AUTH_APPLICATION_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::Repository;
use rocket::form::Form;
use rocket::http::uri::{Origin, Uri};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use url::Url;

pub fn get_authorization_routes() -> Vec<Route> {
    routes![authorize, authorize_guest, authorize_form, single_use_token,]
}

#[get("/auth/authorize?<redirect>", rank = 1)]
pub async fn authorize(
    cookies: &CookieJar<'_>,
    user: User,
    redirect: RedirectUrl,
    repository: &State<ApplicationRepository>,
) -> Result<Template, Status> {
    let key = ModelKey::new_uuid_v8(
        AUTH_APPLICATION_STREAM,
        AUTH_APPLICATION_UUID,
        redirect.url().as_str(),
    );
    let application = repository.get_model(&key).await.unwrap();

    if *application.state() == AuthApplicationState::default() {
        return Err(Status::NotFound);
    }

    let mut session = get_session(cookies);
    session.set_redirect_url(redirect.url());
    session.set_application(Some(key));
    set_session(cookies, session);

    let accounts: Vec<&str> = vec![];

    Ok(Template::render(
        "authorize",
        context! {
            application_name: application.state().name(),
            accounts: accounts,
        },
    ))
}

#[get("/auth/authorize?<redirect>", rank = 2)]
pub async fn authorize_guest(
    cookies: &CookieJar<'_>,
    redirect: RedirectUrl,
    uri: &Origin<'_>,
) -> Result<Redirect, Status> {
    let mut session = get_session(cookies);
    let redirect = format!("{}{}", base_host(), uri.to_string());
    let url = Url::parse(&redirect).map_err(|e| {
        dbg!(e);
        Status::BadRequest
    })?;
    session.set_redirect_url(url);
    dbg!(&session.redirect_url().to_string());
    set_session(cookies, session);

    Ok(Redirect::to(uri!("/login")))
}

#[derive(FromForm, Debug)]
pub struct Authorize {
    account: String,
    new_account: String,
}

#[post("/auth/authorize", data = "<authorize>")]
pub async fn authorize_form(
    cookies: &CookieJar<'_>,
    authorize: Form<Authorize>,
    repository_apps: &State<ApplicationRepository>,
    repository_user: &State<AuthUserRepository>,
    user: User,
) -> Result<Redirect, Status> {
    dbg!(&authorize);

    let mut session = get_session(cookies);
    let Some(app_key) = session.application() else {
        return Err(Status::NotFound);
    };

    let application = repository_apps.get_model(app_key).await.unwrap();
    let user = repository_user.get_model(&user.data().user_id).await.unwrap();

    if authorize.account.is_empty(){
        todo!("create account");
    }else{
        todo!("load account")
    }

    todo!("generate one_time");

    // Ok(Redirect::to(format!(
    //     "{}/auth?token={one_time}",
    //     application.state().host()
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
