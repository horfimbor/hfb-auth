#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::Event;
#[cfg(feature = "server")]
use horfimbor_eventsource::{Event, EventName};

pub const PUB_AUTH_EVENT: &str = "PUB_AUTH_EVENT";
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(PUB_AUTH_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PubAuthEvent {
    UserCreated { pseudo: String },
    AccountCreated { name: String },
    AccountDeleted,
}
