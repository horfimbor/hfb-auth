#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event};
#[cfg(feature = "server")]
use horfimbor_eventsource::{Command, CommandName, Event, EventName};

#[cfg(feature = "server")]
use crate::AUTH_APPLICATION_EVENT;
use auth_public_event::PubAuthEvent;
use serde::{Deserialize, Serialize};
use url::Host;
use uuid::Uuid;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationCommand {
    Create {
        uuid: Uuid,
        name: String,
        host: Host,
        key: String,
    },
    Delete,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_APPLICATION_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PubAuthApplicationEvent {
    Authenticate,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthApplicationEvent {
    Private(PubAuthApplicationEvent),
    Public(PubAuthEvent),
}
