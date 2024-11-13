#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};
#[cfg(feature = "server")]
use horfimbor_eventsource::{Command, CommandName, Event, EventName, StateName, StateNamed};
use horfimbor_eventsource::{Dto, State};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use thiserror::Error;
use url::Host;

#[cfg(feature = "server")]
use crate::AUTH_APPLICATION_EVENT;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationCommand {
    Create {
        name: String,
        host: Host,
        key: String,
    },
    Delete,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrivateAuthApplicationEvent {
    Created {
        name: String,
        host: Host,
        key: String,
    },
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationEvent {
    Private(PrivateAuthApplicationEvent),
}

#[cfg_attr(feature = "server", derive(StateNamed))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq)]
pub struct AuthApplicationState {
    name: String,
    host: Host,
    key: String,
}

impl Default for AuthApplicationState {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            host: Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            key: "".to_string(),
        }
    }
}

impl AuthApplicationState {
    fn play_private_event(&mut self, event: &PrivateAuthApplicationEvent) {
        match event {
            PrivateAuthApplicationEvent::Created { host, key, name } => {
                self.host = host.to_owned();
                self.key = key.clone();
                self.name = name.clone();
            }
        }
    }
}

#[cfg(feature = "server")]
impl Dto for AuthApplicationState {
    type Event = AuthApplicationEvent;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            AuthApplicationEvent::Private(e) => self.play_private_event(e),
        }
    }
}

#[derive(Error, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationError {

}

#[cfg(feature = "server")]
impl State for AuthApplicationState {
    type Command = AuthApplicationCommand;
    type Error = AuthApplicationError;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            AuthApplicationCommand::Create { key, name, host } => {
                Ok(vec![AuthApplicationEvent::Private(
                    PrivateAuthApplicationEvent::Created { key, name, host },
                )])
            }
            AuthApplicationCommand::Delete => {
                // FIXME
                Ok(vec![])
            }
        }
    }
}
