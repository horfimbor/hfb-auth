use crate::admin::application;
use crate::session::{get_session, LoggedInUser};
use crate::AuthUserRepository;
use hfb_auth_shared::user::{UserState, UserRole};
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::{ModelWithPosition, Repository};
use horfimbor_eventsource::EventSourceError;
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Route};
use rocket_dyn_templates::{context, Template};
use std::convert::Infallible;
use std::str::RMatches;

pub fn get_user_routes() -> Vec<Route> {
    routes![index]
}

#[get("/me")]
pub async fn index(user: User) -> Template {
    Template::render(
        "user",
        context! {
            account: user.data(),
            is_admin: user.data().is_admin,
            uri_applications: uri!(application::list)
        },
    )
}

pub struct User {
    data: LoggedInUser,
}

impl User {
    pub fn data(&self) -> &LoggedInUser {
        &self.data
    }
}

pub struct Admin {
    data: LoggedInUser,
}

impl Admin {
    pub fn data(&self) -> &LoggedInUser {
        &self.data
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(data) = get_session(request.cookies()).user() {
            Outcome::Success(User { data: data.clone() })
        } else {
            Outcome::Forward(Default::default())
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(data) = get_session(request.cookies()).user() {
            match data.is_admin {
                true => Outcome::Success(Admin { data: data.clone() }),
                false => Outcome::Forward(Default::default()),
            }
        } else {
            Outcome::Forward(Default::default())
        }
    }
}
