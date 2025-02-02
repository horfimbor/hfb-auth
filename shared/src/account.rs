#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};
#[cfg(feature = "server")]
use horfimbor_eventsource::{
    Command, CommandName, Dto, Event, EventName, State, StateName, StateNamed,
};

use public_account_event::PubAccountEvent;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::AUTH_ACCOUNT_EVENT;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_ACCOUNT_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AccountCommand {
    Create {
        user_id: Uuid,
        app_id: Uuid,
        name: String,
    },
    Validate,
    Suspend,
    Resume,
}

#[derive(Error, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AccountError {
    #[error("Account was already suspended")]
    AlreadySuspended,

    #[error("Account was not suspended")]
    NotSuspended,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_ACCOUNT_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrvAccountEvent {
    Created {
        user_id: Uuid,
        app_id: Uuid,
        name: String,
    },
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AccountEvent {
    Private(PrvAccountEvent),
    Public(PubAccountEvent),
}

#[cfg_attr(feature = "server", derive(StateNamed))]
#[cfg_attr(feature = "server", state(AUTH_ACCOUNT_EVENT))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default, Eq)]
pub struct AccountState {
    user_id: Uuid,
    app_id: Uuid,
    name: String,
    suspend: bool,
}

impl AccountState {
    fn play_private_event(&mut self, event: &PrvAccountEvent) {
        match event {
            PrvAccountEvent::Created {
                app_id,
                user_id,
                name,
            } => {
                self.app_id = app_id.clone();
                self.user_id = user_id.clone();
                self.name = name.clone();
            }
        }
    }
    fn play_public_event(&mut self, event: &PubAccountEvent) {
        match event {
            PubAccountEvent::AccountCreated { .. } => {}
            PubAccountEvent::AccountSuspended => self.suspend = true,
            PubAccountEvent::AccountResumed => self.suspend = false,
        }
    }
}

#[cfg(feature = "server")]
impl Dto for AccountState {
    type Event = AccountEvent;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            AccountEvent::Private(private) => self.play_private_event(private),
            AccountEvent::Public(public) => self.play_public_event(public),
        }
    }
}

#[cfg(feature = "server")]
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
                // TODO check previously exist ?!
                Ok(vec![AccountEvent::Private(PrvAccountEvent::Created {
                    user_id: user_id.clone(),
                    app_id: app_id.clone(),
                    name: name.clone(),
                })])
            }
            AccountCommand::Validate => Ok(vec![AccountEvent::Public(
                PubAccountEvent::AccountCreated {
                    app_id: self.app_id,
                    name: self.name.clone(),
                },
            )]),
            AccountCommand::Suspend => {
                if self.suspend {
                    Err(AccountError::AlreadySuspended)
                } else {
                    Ok(vec![AccountEvent::Public(
                        PubAccountEvent::AccountSuspended,
                    )])
                }
            }
            AccountCommand::Resume => {
                if self.suspend {
                    Ok(vec![AccountEvent::Public(PubAccountEvent::AccountResumed)])
                } else {
                    Err(AccountError::NotSuspended)
                }
            }
        }
    }
}
