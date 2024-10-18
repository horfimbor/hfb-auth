use rocket::form::error::ErrorKind;
use rocket::form::{DataField, Errors, FromFormField, ValueField};
use rocket::http::uri::fmt::Query;
use rocket::http::uri::fmt::{Formatter, FromUriParam, UriDisplay};
use rocket::response::status::BadRequest;
use url::{Host, Url};

#[derive(Debug)]
pub struct RedirectUrl {
    url: Url,
    host: Host<String>,
}

impl RedirectUrl {
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn host(&self) -> Host<String> {
        self.host.clone()
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
                url,
                host: Host::Domain(s.to_string()),
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

    fn from_uri_param(s: String) -> RedirectUrl {
        let url = Url::parse(&*s).unwrap();
        let host = if let Host::Domain(d) = url.host().unwrap() {
            Host::Domain(d.to_string())
        } else {
            todo!("handle case ?")
        };

        RedirectUrl { url, host }
    }
}

impl UriDisplay<Query> for RedirectUrl {
    fn fmt(&self, f: &mut Formatter<'_, Query>) -> std::fmt::Result {
        f.write_raw("redirect:")?;
        UriDisplay::fmt(&self.url.as_str(), f)
    }
}
