use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event, StateNamed};

use horfimbor_eventsource::{Command, CommandName, Event, EventName, StateName, StateNamed};
use horfimbor_eventsource::{Dto, State};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter;
use thiserror::Error;
use url::Url;

use crate::AUTH_APPLICATION_EVENT;

#[derive(Command)]
#[state(AUTH_APPLICATION_EVENT)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ApplicationCommand {
    Create { name: String, host: Url },
    RegenerateKey,
}

#[derive(Event)]
#[state(AUTH_APPLICATION_EVENT)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrivateApplicationEvent {
    Created {
        name: String,
        host: Url,
        key: String,
    },
    KeyChanged {
        key: String,
    },
}

#[derive(Event)]
#[composite_state]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ApplicationEvent {
    Private(PrivateApplicationEvent),
}

#[derive(StateNamed)]
#[state(AUTH_APPLICATION_EVENT)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq)]
pub struct ApplicationState {
    name: String,
    host: Url,
    key: String,
    // TODO ? use 2 key : one to signe, and one for server2server call
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationList {
    pub id: String,
    pub name: String,
    pub host: Url,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            #[allow(clippy::expect_used)]
            host: Url::parse("https://aedius.fr").expect("default url for application is invalid"),
            key: "".to_string(),
        }
    }
}

impl ApplicationState {
    fn play_private_event(&mut self, event: &PrivateApplicationEvent) {
        match event {
            PrivateApplicationEvent::Created { host, key, name } => {
                self.host = host.to_owned();
                self.key = key.clone();
                self.name = name.clone();
            }
            PrivateApplicationEvent::KeyChanged { key } => {
                self.key = key.clone();
            }
        }
    }

    fn generate_key() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::rng();
        let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(40).collect()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn host(&self) -> &Url {
        &self.host
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

impl Dto for ApplicationState {
    type Event = ApplicationEvent;

    fn play_event(&mut self, event: &Self::Event) {
        match event {
            ApplicationEvent::Private(e) => self.play_private_event(e),
        }
    }
}

#[derive(Error, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationError {}

impl State for ApplicationState {
    type Command = ApplicationCommand;
    type Error = AuthApplicationError;

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            ApplicationCommand::Create { name, host } => {
                let key = ApplicationState::generate_key();

                Ok(vec![ApplicationEvent::Private(
                    PrivateApplicationEvent::Created { key, name, host },
                )])
            }
            ApplicationCommand::RegenerateKey => {
                let key = ApplicationState::generate_key();

                Ok(vec![ApplicationEvent::Private(
                    PrivateApplicationEvent::KeyChanged { key },
                )])
            }
        }
    }
}
