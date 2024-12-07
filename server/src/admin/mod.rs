pub mod application;

use crate::user::Admin;
use crate::ApplicationRepository;
use hfb_auth_shared::application::AuthApplicationCommand;
use hfb_auth_shared::AUTH_APPLICATION_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use url::Host;
use uuid::Uuid;

pub fn get_admin_routes() -> Vec<Route> {
    routes![
        admin,
        application::list,
        application::get,
        application::update,
        application::create,
    ]
}

#[get("/admin")]
pub async fn admin(admin: Admin) -> Template {
    Template::render("admin/index", context! {})
}
