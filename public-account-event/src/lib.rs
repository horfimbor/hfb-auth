#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::Event;
#[cfg(feature = "server")]
use horfimbor_eventsource::{Event, EventName};

pub const PUB_ACCOUNT_EVENT: &str = "PUB_ACCOUNT_EVENT";
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(PUB_ACCOUNT_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PubAccountEvent {
    AccountCreated { app_id: Uuid, name: String },
    AccountSuspended,
    AccountResumed,
}
