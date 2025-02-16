use crate::constants::COOKIE_SESSION;
use crate::other_error_page;
use crate::url_parsing::RedirectUrl;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use futures::{FutureExt, TryFutureExt};
use hfb_auth_shared::user::UserRole;
use horfimbor_eventsource::model_key::ModelKey;
use rand::Rng;
use rocket::http::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::{env, iter};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoggedInUser {
    pub user_id: ModelKey,
    pub pseudo: String,
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Csrf {
    value: String,
    expire: DateTime<Utc>,
    form: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionData {
    application: Option<ModelKey>,
    redirect_url: Url,
    user: Option<LoggedInUser>,
    csrf: Option<Csrf>, // TODO use in each form
}

pub fn base_host() -> Result<String> {
    env::var("APP_HOST").context("APP_HOST was defined at launch.")
}

impl SessionData {
    pub fn redirect_url(&self) -> &Url {
        &self.redirect_url
    }

    pub fn set_redirect_url(&mut self, redirect_url: Url) {
        self.redirect_url = redirect_url;
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

    pub fn csrf(&self) -> &Option<Csrf> {
        &self.csrf
    }

    pub fn set_csrf(&mut self, csrf: Option<Csrf>) {
        self.csrf = csrf;
    }

    pub fn check_csrf(&self, form: &'static str, value: &str) -> Result<()> {
        match self.csrf() {
            None => {
                bail!("no csrf");
            }
            Some(csrf) => {
                if !csrf.check(form, value) {
                    bail!("wrong csrf");
                }
            }
        }

        Ok(())
    }
}

pub fn get_session(cookies: &CookieJar<'_>) -> Result<SessionData> {
    let from_cookie =
        cookies.get_private(COOKIE_SESSION).and_then(|data| {
            match serde_json::from_str::<SessionData>(data.value()) {
                Ok(d) => Some(d),
                Err(e) => {
                    dbg!(e);
                    None
                }
            }
        });
    match from_cookie {
        None => {
            let auth_host = base_host()?;
            Ok(SessionData {
                application: None,
                redirect_url: Url::parse(&auth_host).context("APP_HOST must be a valid url")?,
                user: None,
                csrf: None,
            })
        }
        Some(c) => Ok(c),
    }
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

impl Csrf {
    pub fn new(form: &'static str) -> Self {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::rng();
        let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
        let value = iter::repeat_with(one_char).take(40).collect();

        let expire = Utc::now() + Duration::minutes(10);

        Csrf {
            value,
            expire,
            form: form.to_string(),
        }
    }

    fn check(&self, form: &'static str, value: &str) -> bool {
        if self.form != form {
            return false;
        }
        let now = Utc::now();
        if now > self.expire {
            return false;
        }

        self.value == value
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
