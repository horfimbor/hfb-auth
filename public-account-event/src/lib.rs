use horfimbor_eventsource::model_key::ModelKey;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use horfimbor_eventsource::horfimbor_eventsource_derive::Event;
#[cfg(feature = "server")]
use horfimbor_eventsource::{Event, EventName};
#[cfg(feature = "server")]
pub const PUB_ACCOUNT_EVENT: &str = "PUB_ACCOUNT_EVENT";

#[cfg_attr(feature = "server", derive(Event))]
#[cfg_attr(feature = "server", state(PUB_ACCOUNT_EVENT))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PubAccountEvent {
    AccountCreated {
        user_id: ModelKey,
        app_id: ModelKey,
        name: String,
    },
    AccountSuspended,
    AccountResumed,
}
