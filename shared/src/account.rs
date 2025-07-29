use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};
use horfimbor_eventsource::model_key::ModelKey;
use std::iter;

use horfimbor_eventsource::{
    Command, CommandName, Dto, Event, EventName, State, StateName, StateNamed,
};
use public_account_event::PubAccountEvent;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::AUTH_ACCOUNT_EVENT;

#[derive(Command)]
#[state(AUTH_ACCOUNT_EVENT)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AccountCommand {
    Create {
        user_id: ModelKey,
        app_id: ModelKey,
        name: String,
    },
    Validate,
    Suspend,
    Resume,
    NewOneTimeToken,
}

#[derive(Error, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AccountError {
    #[error("Account was already suspended")]
    AlreadySuspended,

    #[error("Account was already created")]
    AlreadyCreated,

    #[error("Account was not suspended")]
    NotSuspended,

    #[error("No one time code")]
    NoOneTimeCode,
}

#[derive(Event)]
#[state(AUTH_ACCOUNT_EVENT)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrvAccountEvent {
    Created {
        user_id: ModelKey,
        app_id: ModelKey,
        name: String,
    },
    OneTimeTokenAdded {
        token: String,
    },
    OneTimeTokenRemoved,
}

#[derive(Event)]
#[composite_state]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AccountEvent {
    Private(PrvAccountEvent),
    Public(PubAccountEvent),
}

#[derive(StateNamed)]
#[state(AUTH_ACCOUNT_EVENT)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default, Eq)]
pub struct AccountState {
    user_id: ModelKey,
    app_id: ModelKey,
    name: String,
    suspended: bool,
    one_time_token: Option<String>,
    validated: bool,
}

impl AccountState {
    fn play_private_event(&mut self, event: &PrvAccountEvent) {
        match event {
            PrvAccountEvent::Created {
                user_id,
                app_id,
                name,
            } => {
                self.app_id = app_id.clone();
                self.user_id = user_id.clone();
                self.name = name.clone();
                self.suspended = true;
            }
            PrvAccountEvent::OneTimeTokenAdded { token } => {
                self.one_time_token = Some(token.clone())
            }
            PrvAccountEvent::OneTimeTokenRemoved => self.one_time_token = None,
        }
    }

    fn generate_one_time_token() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::rng();
        let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(20).collect()
    }

    fn play_public_event(&mut self, event: &PubAccountEvent) {
        match event {
            PubAccountEvent::AccountCreated { .. } => self.validated = true,
            PubAccountEvent::AccountSuspended => self.suspended = true,
            PubAccountEvent::AccountResumed => self.suspended = false,
        }
    }

    pub fn one_time_token(&self, key: &ModelKey) -> Result<String, AccountError> {
        match self.one_time_token.clone() {
            None => Err(AccountError::NoOneTimeCode),
            Some(otk) => Ok(format!("{key}|{otk}")),
        }
    }

    pub fn user_id(&self) -> &ModelKey {
        &self.user_id
    }

    pub fn app_id(&self) -> &ModelKey {
        &self.app_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Dto for AccountState {
    type Event = AccountEvent;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            AccountEvent::Private(private) => self.play_private_event(private),
            AccountEvent::Public(public) => self.play_public_event(public),
        }
    }
}

impl State for AccountState {
    type Command = AccountCommand;
    type Error = AccountError;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            AccountCommand::Create {
                name,
                app_id,
                user_id,
            } => {
                if self.app_id != ModelKey::default() {
                    return Err(AccountError::AlreadyCreated);
                }

                Ok(vec![
                    AccountEvent::Private(PrvAccountEvent::Created {
                        user_id: user_id.clone(),
                        app_id: app_id.clone(),
                        name: name.clone(),
                    }),
                    AccountEvent::Private(PrvAccountEvent::OneTimeTokenAdded {
                        token: Self::generate_one_time_token(),
                    }),
                ])
            }
            AccountCommand::Validate => {
                let mut events = vec![
                    AccountEvent::Public(PubAccountEvent::AccountResumed),
                    AccountEvent::Private(PrvAccountEvent::OneTimeTokenRemoved),
                ];

                if !self.validated {
                    events.push(AccountEvent::Public(PubAccountEvent::AccountCreated {
                        user_id: self.user_id.clone(),
                        app_id: self.app_id.clone(),
                        name: self.name.clone(),
                    }))
                }

                Ok(events)
            }
            AccountCommand::Suspend => {
                if self.suspended {
                    Err(AccountError::AlreadySuspended)
                } else {
                    Ok(vec![AccountEvent::Public(
                        PubAccountEvent::AccountSuspended,
                    )])
                }
            }
            AccountCommand::Resume => {
                if self.suspended {
                    Ok(vec![AccountEvent::Public(PubAccountEvent::AccountResumed)])
                } else {
                    Err(AccountError::NotSuspended)
                }
            }
            AccountCommand::NewOneTimeToken => Ok(vec![AccountEvent::Private(
                PrvAccountEvent::OneTimeTokenAdded {
                    token: Self::generate_one_time_token(),
                },
            )]),
        }
    }
}
