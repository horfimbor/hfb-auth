use crate::admin::application;
use crate::session::get_session;
use crate::AuthUserRepository;
use hfb_auth_shared::user::{AuthUserState, UserRole};
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
            account: user.state(),
            is_admin: user.state().is_admin(),
            uri_applications: uri!(application::list)
        },
    )
}

pub struct User {
    state: AuthUserState,
}

impl User {
    pub fn state(&self) -> &AuthUserState {
        &self.state
    }
}

pub struct Admin {
    state: AuthUserState,
}

impl Admin {
    pub fn state(&self) -> &AuthUserState {
        &self.state
    }
}

#[derive(Debug)]
pub enum UserErr {
    NoCookie,
    NoRepository,
    CannotParseModelKey,
    NotFound,
    DBError,
    AccessRefused,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserErr;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(data) = get_session(request.cookies()).user() {
            if let Some(repo) = request.rocket().state::<AuthUserRepository>() {
                match load(data.user_id.clone(), repo, UserRole::None).await {
                    Ok(state) => Outcome::Success(User { state }),
                    Err(e) => Outcome::Error(e),
                }
            } else {
                Outcome::Error((Status::InternalServerError, UserErr::NoRepository))
            }
        } else {
            Outcome::Forward(Default::default())
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = UserErr;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(data) = get_session(request.cookies()).user() {
            if let Some(repo) = request.rocket().state::<AuthUserRepository>() {
                match load(data.user_id.clone(), repo, UserRole::Admin).await {
                    Ok(state) => Outcome::Success(Admin { state }),
                    Err(e) => Outcome::Error(e),
                }
            } else {
                Outcome::Error((Status::InternalServerError, UserErr::NoRepository))
            }
        } else {
            Outcome::Error((Status::Forbidden, UserErr::NoCookie))
        }
    }
}

async fn load(
    key: String,
    repo: &AuthUserRepository,
    role: UserRole,
) -> Result<AuthUserState, (Status, UserErr)> {
    let mk: Result<ModelKey, _> = ModelKey::try_from(key.as_str());
    match mk {
        Ok(mk) => {
            let user = repo.get_model(&mk).await;

            match user {
                Ok(u) => match role {
                    UserRole::Admin => {
                        if u.state().is_admin() {
                            Ok(u.state().clone())
                        } else {
                            Err((Status::Forbidden, UserErr::AccessRefused))
                        }
                    }
                    UserRole::None => Ok(u.state().clone()),
                },
                Err(err) => match err {
                    EventSourceError::Uuid(_) => Err((Status::NotFound, UserErr::DBError)),
                    _ => Err((Status::InternalServerError, UserErr::NoRepository)),
                },
            }
        }
        Err(_) => Err((Status::InternalServerError, UserErr::CannotParseModelKey)),
    }
}
