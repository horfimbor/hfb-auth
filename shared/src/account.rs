#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::{Command, Event};
#[cfg(feature = "server")]
use horfimbor_eventsource::{Command, CommandName, Event, EventName};

use public_user_event::PubUserEvent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::AUTH_ACCOUNT_EVENT;

#[cfg_attr(feature = "server", derive(Command))]
#[cfg_attr(feature = "server", state(AUTH_ACCOUNT_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthAccountCommand {
    Create,
    Auth,
    Delete,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(AUTH_ACCOUNT_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PubAuthAccountEvent {
    Authenticate,
}

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", composite_state)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthAccountEvent {
    Private(PubAuthAccountEvent),
    Public(PubUserEvent),
}
