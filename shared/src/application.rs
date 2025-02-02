#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};
#[cfg(feature = "server")]
use horfimbor_eventsource::{Command, CommandName, Event, EventName, StateName, StateNamed};
use horfimbor_eventsource::{Dto, State};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter;
use thiserror::Error;
use url::Url;

#[cfg(feature = "server")]
use crate::AUTH_APPLICATION_EVENT;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationCommand {
    Create { name: String, host: Url },
    RegenerateKey,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrivateAuthApplicationEvent {
    Created {
        name: String,
        host: Url,
        key: String,
    },
    KeyChanged {
        key: String,
    },
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AuthApplicationEvent {
    Private(PrivateAuthApplicationEvent),
}

#[cfg_attr(feature = "server", derive(StateNamed))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq)]
pub struct AuthApplicationState {
    name: String,
    host: Url,
    key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthApplicationList {
    pub id: String,
    pub name: String,
    pub host: Url,
}

impl Default for AuthApplicationState {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            host: Url::parse("https://aedius.fr").expect("default url for application is invalid"),
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
            PrivateAuthApplicationEvent::KeyChanged { key } => {
                self.key = key.clone();
            }
        }
    }

    fn generate_key() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::thread_rng();
        let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(40).collect()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn host(&self) -> &Url {
        &self.host
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
pub enum AuthApplicationError {}

#[cfg(feature = "server")]
impl State for AuthApplicationState {
    type Command = AuthApplicationCommand;
    type Error = AuthApplicationError;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            AuthApplicationCommand::Create { name, host } => {
                let key = AuthApplicationState::generate_key();

                Ok(vec![AuthApplicationEvent::Private(
                    PrivateAuthApplicationEvent::Created { key, name, host },
                )])
            }
            AuthApplicationCommand::RegenerateKey => {
                let key = AuthApplicationState::generate_key();

                Ok(vec![AuthApplicationEvent::Private(
                    PrivateAuthApplicationEvent::KeyChanged { key },
                )])
            }
        }
    }
}
