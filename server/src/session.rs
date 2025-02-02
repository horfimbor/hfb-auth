use crate::constants::COOKIE_SESSION;
use crate::url_parsing::RedirectUrl;
use anyhow::Result;
use futures::{FutureExt, TryFutureExt};
use hfb_auth_shared::user::UserRole;
use horfimbor_eventsource::model_key::ModelKey;
use rocket::http::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoggedInUser {
    pub user_id: ModelKey,
    pub pseudo: String,
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionData {
    application: Option<ModelKey>,
    error: Option<SessionError>,
    redirect_url: Url,
    user: Option<LoggedInUser>,
    // csrf: TODO
}

pub fn base_host() -> String {
    env::var("APP_HOST").expect("APP_HOST was defined at launch.")
}

impl Default for SessionData {
    fn default() -> Self {
        let auth_host = base_host();
        Self {
            user: None,
            redirect_url: Url::parse(&auth_host).expect("APP_HOST must be a valid url"),
            error: None,
            application: None,
        }
    }
}

impl SessionData {
    pub fn redirect_url(&self) -> &Url {
        &self.redirect_url
    }

    pub fn set_redirect_url(&mut self, redirect_url: Url) {
        self.redirect_url = redirect_url;
    }

    pub fn error(&self) -> &Option<SessionError> {
        &self.error
    }

    pub fn set_error(&mut self, error: Option<SessionError>) {
        self.error = error;
    }

    pub fn user(&self) -> &Option<LoggedInUser> {
        &self.user
    }

    pub fn set_user(&mut self, user: Option<LoggedInUser>) {
        self.user = user;
    }

    pub fn application(&self) -> &Option<ModelKey> {
        &self.application
    }

    pub fn set_application(&mut self, application: Option<ModelKey>) {
        self.application = application;
    }
}

pub fn get_session(cookies: &CookieJar<'_>) -> SessionData {
    cookies
        .get_private(COOKIE_SESSION)
        .and_then(
            |data| match serde_json::from_str::<SessionData>(data.value()) {
                Ok(d) => Some(d),
                Err(e) => {
                    dbg!(e);
                    None
                }
            },
        )
        .unwrap_or_default()
}

pub fn set_session(cookies: &CookieJar<'_>, data: SessionData) -> Result<()> {
    let mut cookie = Cookie::new(COOKIE_SESSION, serde_json::to_string(&data)?);
    cookie.set_same_site(Some(SameSite::Lax));
    cookies.add_private(cookie);
    Ok(())
}

pub fn remove_session(cookies: &CookieJar<'_>) {
    cookies.remove(COOKIE_SESSION);
}
