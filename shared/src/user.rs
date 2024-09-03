#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};
#[cfg(feature = "server")]
use horfimbor_eventsource::{
    Command, CommandName, Dto, Event, EventName, State, StateName, StateNamed,
};

#[cfg(feature = "server")]
use crate::AUTH_USER_EVENT;

use auth_public_event::PubAuthEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_USER_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthUserCommand {
    Create {
        pseudo: String,
        password_hash: String,
    },
    ChangePassword {
        password_hash: String,
    },
    Login,
}

#[derive(Error, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UserError {
    #[error("Login failed")]
    LoginFail,
    #[error("An account already exists")]
    AlreadyExists,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_USER_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrvAuthUserEvent {
    Created {
        date: DateTime<Utc>,
    },
    PasswordChanged {
        password_hash: String,
        date: DateTime<Utc>,
    },
    LoggedIn(DateTime<Utc>),
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum AuthUserEvent {
    Private(PrvAuthUserEvent),
    Public(PubAuthEvent),
}

#[cfg_attr(feature = "server", derive(StateNamed))]
#[cfg_attr(feature = "server", state(AUTH_USER_EVENT))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default, Eq)]
pub struct AuthUserState {
    pseudo: String,
    password_hash: String,
    password_change: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
}

impl AuthUserState {
    pub fn play_event(&mut self, event: &AuthUserEvent) {
        match event {
            AuthUserEvent::Private(private) => {
                self.play_private_event(private);
            }
            AuthUserEvent::Public(public) => {
                self.play_public_event(public);
            }
        }
    }

    fn play_private_event(&mut self, event: &PrvAuthUserEvent) {
        match event {
            PrvAuthUserEvent::Created { date } => {
                self.created_at = date.clone();
            }
            PrvAuthUserEvent::PasswordChanged {
                password_hash,
                date,
            } => {
                self.password_hash = password_hash.clone();
                self.password_change = date.clone();
            }
            PrvAuthUserEvent::LoggedIn(date) => {
                self.last_login = Some(date.clone());
            }
        }
    }
    fn play_public_event(&mut self, event: &PubAuthEvent) {
        match event {
            PubAuthEvent::UserCreated { pseudo } => {
                self.pseudo = pseudo.clone();
            }
            PubAuthEvent::AccountCreated { .. } => {}
            PubAuthEvent::AccountDeleted => {}
        }
    }
}

#[cfg(feature = "server")]
impl Dto for AuthUserState {
    type Event = AuthUserEvent;

    fn play_event(&mut self, event: &Self::Event) {
        self.play_event(event);
    }
}

#[cfg(feature = "server")]
impl State for AuthUserState {
    type Command = AuthUserCommand;
    type Error = UserError;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            AuthUserCommand::Create {
                pseudo,
                password_hash,
            } => {
                let default = Self::default();
                if *self != default {
                    return Err(UserError::AlreadyExists);
                }

                let date = Utc::now();
                Ok(vec![
                    AuthUserEvent::Public(PubAuthEvent::UserCreated { pseudo }),
                    AuthUserEvent::Private(PrvAuthUserEvent::Created { date }),
                    AuthUserEvent::Private(PrvAuthUserEvent::PasswordChanged {
                        date,
                        password_hash,
                    }),
                ])
            }
            AuthUserCommand::ChangePassword { .. } => {
                todo!()
            }
            AuthUserCommand::Login => {
                todo!()
            }
        }
    }
}
