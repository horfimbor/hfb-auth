use crate::constants::COOKIE_SESSION;
use crate::AuthUserRepository;
use hfb_auth_shared::user::{AuthUserState, UserRole};
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::{ModelWithPosition, Repository};
use horfimbor_eventsource::EventSourceError;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use std::convert::Infallible;
use std::str::RMatches;

pub struct User {
    state: AuthUserState,
}

pub struct Admin {
    state: AuthUserState,
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
        if let Some(key) = request
            .cookies()
            .get_private(COOKIE_SESSION)
            .map(|v| v.value().to_string())
        {
            if let Some(repo) = request.rocket().state::<AuthUserRepository>() {
                match load(key, repo, UserRole::None).await {
                    Ok(state) => Outcome::Success(User { state }),
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = UserErr;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(key) = request
            .cookies()
            .get_private(COOKIE_SESSION)
            .map(|v| v.value().to_string())
        {
            if let Some(repo) = request.rocket().state::<AuthUserRepository>() {
                match load(key, repo, UserRole::Admin).await {
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
