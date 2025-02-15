use core::fmt;
use rocket::form::error::ErrorKind;
use rocket::form::{DataField, Errors, FromFormField, ValueField};
use rocket::http::uri::fmt::Query;
use rocket::http::uri::fmt::{Formatter, FromUriParam, UriDisplay};
use rocket::response::status::BadRequest;
use serde::{Deserialize, Serialize};
use url::{Host, ParseError, Url};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedirectUrl {
    url: Option<Url>,
    #[serde(skip)]
    error: Option<ParseError>,
}

impl RedirectUrl {
    pub fn url(&self) -> Result<Url, ParseError> {
        match self.url.clone() {
            Some(url) => Ok(url),
            None => match self.error {
                None => Err(ParseError::EmptyHost),
                Some(e) => Err(e),
            },
        }
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for RedirectUrl {
    fn from_value(field: ValueField<'r>) -> rocket::form::Result<'r, Self> {
        let url = Url::parse(field.value)
            .map_err(|e| ErrorKind::Validation(format!("invalid url {e}").into()))?;

        let binding = url.clone();
        let host = binding
            .host()
            .ok_or(ErrorKind::Validation("no host".into()))?;
        match host {
            Host::Domain(s) => Ok(RedirectUrl {
                url: Some(url),
                error: None,
            }),
            Host::Ipv4(_) => Err(Errors::from(rocket::form::Error::validation(
                "domain cannot be an ip v4",
            ))),
            Host::Ipv6(_) => Err(Errors::from(rocket::form::Error::validation(
                "domain cannot be an ip v6",
            ))),
        }
    }

    async fn from_data(field: DataField<'r, '_>) -> rocket::form::Result<'r, Self> {
        todo!()
    }
}

impl FromUriParam<Query, String> for RedirectUrl {
    type Target = RedirectUrl;

    fn from_uri_param(s: String) -> Self::Target {
        let url = Url::parse(&s);

        match url {
            Ok(url) => RedirectUrl {
                url: Some(url),
                error: None,
            },
            Err(error) => RedirectUrl {
                url: None,
                error: Some(error),
            },
        }
    }
}

impl UriDisplay<Query> for RedirectUrl {
    fn fmt(&self, f: &mut Formatter<'_, Query>) -> std::fmt::Result {
        f.write_raw("redirect:")?;
        match &self.url {
            None => Err(fmt::Error),
            Some(url) => UriDisplay::fmt(url.as_str(), f),
        }
    }
}
