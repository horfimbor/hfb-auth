#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};
#[cfg(feature = "server")]
use horfimbor_eventsource::{
    Command, CommandName, Dto, Event, EventName, State, StateName, StateNamed,
};

#[cfg(feature = "server")]
use crate::AUTH_USER_EVENT;

use chrono::{DateTime, Utc};
use public_user_event::PubUserEvent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_USER_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UserCommand {
    Create {
        pseudo: String,
        password_hash: String,
    },
    ChangePassword {
        password_hash: String,
    },
    Login, // TODO logic must be in command not in controller
    ChangeRole(Option<UserRole>),
}

#[derive(Error, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UserError {
    #[error("Login failed")]
    LoginFail,
    #[error("An user already exists")]
    AlreadyExists,
    #[error("User has already the role")]
    SameRole,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_USER_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrvUserEvent {
    Created {
        date: DateTime<Utc>,
    },
    PasswordChanged {
        password_hash: String,
        date: DateTime<Utc>,
    },
    LoggedIn(DateTime<Utc>),
    ChangeRole(Option<UserRole>),
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum UserEvent {
    Private(PrvUserEvent),
    Public(PubUserEvent),
}

#[cfg_attr(feature = "server", derive(StateNamed))]
#[cfg_attr(feature = "server", state(AUTH_USER_EVENT))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default, Eq)]
pub struct UserState {
    pseudo: String,
    password_hash: Option<String>,
    password_change: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
    role: Option<UserRole>,
}

impl UserState {
    fn play_private_event(&mut self, event: &PrvUserEvent) {
        match event {
            PrvUserEvent::Created { date } => {
                self.created_at = *date;
            }
            PrvUserEvent::PasswordChanged {
                password_hash,
                date,
            } => {
                self.password_hash = Some(password_hash.clone());
                self.password_change = *date;
            }
            PrvUserEvent::LoggedIn(date) => {
                self.last_login = Some(*date);
            }
            PrvUserEvent::ChangeRole(role) => self.role = role.clone(),
        }
    }

    fn play_public_event(&mut self, event: &PubUserEvent) {
        match event {
            PubUserEvent::UserCreated { pseudo } => {
                self.pseudo = pseudo.clone();
            }
        }
    }

    pub fn pseudo(&self) -> &str {
        &self.pseudo
    }

    pub fn password_hash(&self) -> &Option<String> {
        &self.password_hash
    }

    pub fn password_change(&self) -> DateTime<Utc> {
        self.password_change
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn last_login(&self) -> Option<DateTime<Utc>> {
        self.last_login
    }

    pub fn is_admin(&self) -> bool {
        self.role == Some(UserRole::Admin)
    }
}

#[cfg(feature = "server")]
impl Dto for UserState {
    type Event = UserEvent;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            UserEvent::Private(private) => {
                self.play_private_event(private);
            }
            UserEvent::Public(public) => {
                self.play_public_event(public);
            }
        }
    }
}

#[cfg(feature = "server")]
impl State for UserState {
    type Command = UserCommand;
    type Error = UserError;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            UserCommand::Create {
                pseudo,
                password_hash,
            } => {
                let default = Self::default();
                if *self != default {
                    return Err(UserError::AlreadyExists);
                }

                let date = Utc::now();
                Ok(vec![
                    UserEvent::Public(PubUserEvent::UserCreated { pseudo }),
                    UserEvent::Private(PrvUserEvent::Created { date }),
                    UserEvent::Private(PrvUserEvent::PasswordChanged {
                        date,
                        password_hash,
                    }),
                ])
            }
            UserCommand::ChangePassword { password_hash } => {
                let date = Utc::now();
                Ok(vec![UserEvent::Private(PrvUserEvent::PasswordChanged {
                    date,
                    password_hash,
                })])
            }
            UserCommand::Login => Ok(vec![UserEvent::Private(PrvUserEvent::LoggedIn(Utc::now()))]),
            UserCommand::ChangeRole(role) => {
                if role == self.role {
                    return Err(UserError::SameRole);
                }

                Ok(vec![UserEvent::Private(PrvUserEvent::ChangeRole(role))])
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UserRole {
    Admin,
    None,
}

// needed for clap ??!
impl From<String> for UserRole {
    fn from(value: String) -> Self {
        if value == "Admin" {
            Self::Admin
        } else {
            Self::None
        }
    }
}
