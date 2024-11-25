use horfimbor_eventsource::model_key::ModelKey;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

#[get("/administration_panel")]
pub async fn admin(cookies: &CookieJar<'_>) -> Template {
    Template::render("admin/index", context! {})
}

#[derive(FromForm, Debug)]
pub struct Application<'r> {
    name: &'r str,
    host: &'r str
}

#[post("/admin-application", data = "<Application>")]
pub async fn admin_application(
    application : Form<Application<'_>>,
    repository :ApplicationRepository,
    cookies: &CookieJar<'_>) -> Template {

    let key = ModelKey::new_uuid_v8(AUTH_APPLICATION_STREAM,AUTH_APPLICATION_UUID, application.name);

    let model = repository/

    Template::render("admin/index", context! {})

}
