use horfimbor_eventsource::model_key::ModelKey;

pub mod account;
pub mod application;
pub mod user;

pub const AUTH_ACCOUNT_EVENT: &str = "AUTH_ACCOUNT_EVENT";
pub const AUTH_APPLICATION_EVENT: &str = "AUTH_APPLICATION_EVENT";
pub const AUTH_USER_EVENT: &str = "AUTH_USER_EVENT";

pub const AUTH_USER_STREAM: &str = "user_stream";
pub const AUTH_APPLICATION_STREAM: &str = "apps_stream";
pub const AUTH_ACCOUNT_STREAM: &str = "account_stream";

type AccountId = ModelKey;
type ApplicationId = ModelKey;
