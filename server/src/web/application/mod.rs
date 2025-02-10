use crate::constants::AUTH_APPLICATION_UUID;
use crate::session::{base_host, get_session, set_session};
use crate::url_parsing::RedirectUrl;
use crate::user::User;
use crate::web::error::ErrorPage;
use crate::web::public::connexion;
use crate::{other_error, AccountRepository, ApplicationRepository, UserRepository};
use anyhow::anyhow;
use hfb_auth_shared::account::AccountCommand;
use hfb_auth_shared::application::ApplicationState;
use hfb_auth_shared::user::UserCommand;
use hfb_auth_shared::{AUTH_ACCOUNT_STREAM, AUTH_APPLICATION_STREAM};
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::Repository;
use horfimbor_eventsource::State as HorfimborState;
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::form::Form;
use rocket::http::uri::{Origin, Uri};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use uuid::Uuid;

pub fn get_authorization_routes() -> Vec<Route> {
    routes![authorize, authorize_guest, authorize_form, single_use_token,]
}

#[get("/auth/authorize?<redirect>", rank = 1)]
pub async fn authorize(
    cookies: &CookieJar<'_>,
    user: User,
    redirect: RedirectUrl,
    application_repository: &State<ApplicationRepository>,
    repository_user: &State<UserRepository>,
) -> Result<Template, ErrorPage> {
    dbg!(&redirect.url());

    let application_id = ModelKey::new_uuid_v8(
        AUTH_APPLICATION_STREAM,
        AUTH_APPLICATION_UUID,
        redirect.url().as_str(),
    );

    dbg!(&application_id);

    let application = application_repository
        .get_model(&application_id)
        .await
        .map_err(|e| other_error!(e))?;

    if *application.state() == ApplicationState::default() {
        return Err(other_error!("application not found"));
    }

    let mut session = get_session(cookies);
    session.set_redirect_url(redirect.url());
    session.set_application(Some(application_id.clone()));
    set_session(cookies, session);

    let user = repository_user
        .get_model(&user.data().user_id)
        .await
        .map_err(|e| other_error!(e))?;

    dbg!(user.state().accounts(&application_id));

    let accounts: Vec<(String, String)> = user
        .state()
        .accounts(&application_id)
        .into_iter()
        .map(|t| (t.0.format(), t.1))
        .collect();

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
) -> Result<Redirect, ErrorPage> {
    let mut session = get_session(cookies);
    let redirect = format!("{}{}", base_host(), uri);
    let url = Url::parse(&redirect).map_err(|e| other_error!(e))?;
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
    repository_user: &State<UserRepository>,
    repository_account: &State<AccountRepository>,
    user: User,
) -> Result<Redirect, ErrorPage> {
    dbg!(&authorize);

    let mut session = get_session(cookies);
    let Some(app_key) = session.application() else {
        return Err(other_error!("application not found"));
    };

    let application = repository_apps
        .get_model(app_key)
        .await
        .map_err(|e| other_error!(e))?;

    let user_model = repository_user
        .get_model(&user.data().user_id)
        .await
        .map_err(|e| other_error!(e))?;

    let one_time = if authorize.account.is_empty() && !authorize.new_account.is_empty() {
        let new_account_key = ModelKey::new(AUTH_ACCOUNT_STREAM, Uuid::new_v4());

        let model = repository_account
            .add_command(
                &new_account_key,
                AccountCommand::Create {
                    user_id: user.data().user_id.clone(),
                    app_id: app_key.clone(),
                    name: authorize.new_account.clone(),
                },
                None,
            )
            .await
            .map_err(|e| other_error!(e))?;

        let _ = user_model
            .state()
            .try_command(UserCommand::AddAccount {
                application: app_key.clone(),
                account: new_account_key.clone(),
                label: authorize.new_account.clone(),
            })
            .map_err(|e| other_error!(e))?;

        dbg!(&model);

        model.one_time_token(&new_account_key)
    } else if !authorize.account.is_empty() {
        let account_id: ModelKey = authorize
            .account
            .as_str()
            .try_into()
            .map_err(|e: uuid::Error| other_error!(e))?;

        if user_model.state().has_account(app_key, &account_id) {
            let model = repository_account
                .add_command(&account_id, AccountCommand::NewOneTimeToken, None)
                .await
                .map_err(|e| other_error!(e))?;

            model.one_time_token(&account_id)
        } else {
            todo!("error")
        }
    } else {
        todo!("error")
    }
    .map_err(|e| other_error!(e))?;

    Ok(Redirect::to(format!(
        "{}/auth?token={one_time}",
        application.state().host()
    )))
}

#[derive(FromForm, Debug)]
pub struct SingleUseToken<'r> {
    token: &'r str,
    app_key: &'r str,
}

#[post("/auth/single-use-token", data = "<data>")]
pub async fn single_use_token(
    data: Form<SingleUseToken<'_>>,
    repository_apps: &State<ApplicationRepository>,
    repository_user: &State<UserRepository>,
    repository_account: &State<AccountRepository>,
) -> Result<String, ErrorPage> {
    dbg!(&data);

    let mut split = data.token.split('|');
    let key = ModelKey::try_from(split.next().unwrap_or_default()).map_err(|e| other_error!(e))?;

    let account = repository_account
        .get_model(&key)
        .await
        .map_err(|e| other_error!(e))?;

    dbg!(&account.state());

    if account
        .state()
        .one_time_token(&key)
        .map_err(|e| other_error!(e))?
        != data.token
    {
        todo!("wrong one time token")
    }

    let application = repository_apps
        .get_model(account.state().app_id())
        .await
        .map_err(|e| other_error!(e))?;
    let application = application.state();

    dbg!(&application);
    dbg!(&data);

    if application.key() != data.app_key {
        todo!("wrong app key")
    }

    let account = repository_account
        .add_command(&key, AccountCommand::Validate, None)
        .await
        .map_err(|e| other_error!(e))?;

    dbg!(&account);

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| other_error!(e))?
        .as_secs();

    let claims = Claims {
        aud: application
            .name()
            .parse()
            .map_err(|e: Infallible| other_error!(e))?,
        exp: (since_the_epoch + 3600) as usize,
        iat: since_the_epoch as usize,
        iss: "login".parse().map_err(|e: Infallible| other_error!(e))?,
        sub: "user".parse().map_err(|e: Infallible| other_error!(e))?,
        id: key.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(application.key().as_ref()),
    )
    .map_err(|e| other_error!(e))?;

    dbg!(&token);

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
