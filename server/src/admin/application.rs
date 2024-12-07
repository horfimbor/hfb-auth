use crate::constants::AUTH_APPLICATION_UUID;
use crate::ApplicationRepository;
use hfb_auth_shared::application::AuthApplicationCommand;
use hfb_auth_shared::AUTH_APPLICATION_STREAM;
use horfimbor_eventsource::model_key::ModelKey;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use url::Host;

#[derive(FromForm, Debug)]
pub struct Application<'r> {
    name: &'r str,
    host: &'r str,
}

#[get("/admin/applications")]
pub async fn list(cookies: &CookieJar<'_>) -> Template {
    Template::render("admin/applications", context! {})
}

#[get("/admin/application/<id>")]
pub async fn get(cookies: &CookieJar<'_>, id: &str) -> Template {
    Template::render("admin/applications", context! {})
}

#[post("/admin/update-application/<id>", data = "<application>")]
pub async fn update(
    application: Form<Application<'_>>,
    repository: &State<ApplicationRepository>,
    cookies: &CookieJar<'_>,
    id: &str,
) -> Template {
    let key = ModelKey::new(AUTH_APPLICATION_STREAM, id.parse().unwrap());
    todo!("todo update");

    let model = repository
        .add_command(
            &key,
            AuthApplicationCommand::Create {
                name: application.name.to_string(),
                host: Host::parse(application.host).unwrap(),
            },
            None,
        )
        .await
        .unwrap();

    Template::render("admin/index", context! {})
}

#[post("/admin/create-application", data = "<application>")]
pub async fn create(
    application: Form<Application<'_>>,
    repository: &State<ApplicationRepository>,
    cookies: &CookieJar<'_>,
) -> Template {
    let key = ModelKey::new_uuid_v8(
        AUTH_APPLICATION_STREAM,
        AUTH_APPLICATION_UUID,
        application.name,
    );

    let model = repository
        .add_command(
            &key,
            AuthApplicationCommand::Create {
                name: application.name.to_string(),
                host: Host::parse(application.host).unwrap(),
            },
            None,
        )
        .await
        .unwrap();

    Template::render("admin/index", context! {})
}
