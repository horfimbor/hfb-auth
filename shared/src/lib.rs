pub mod account;
pub mod application;
pub mod user;

pub const AUTH_ACCOUNT_EVENT: &str = "AUTH_ACCOUNT_EVENT";
pub const AUTH_APPLICATION_EVENT: &str = "AUTH_APPLICATION_EVENT";
pub const USER_EVENT: &str = "USER_EVENT";

pub const AUTH_USER_STREAM: &str = "user_stream";
pub const AUTH_APPLICATION_STREAM: &str = "apps_stream";
