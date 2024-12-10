use crate::constants::{APPLICATION_LIST_REDIS_KEY, AUTH_APPLICATION_UUID};
use crate::user::Admin;
use crate::ApplicationRepository;
use eventstore::Client;
use hfb_auth_shared::application::{AuthApplicationCommand, PrivateAuthApplicationEvent};
use hfb_auth_shared::AUTH_APPLICATION_STREAM;
use horfimbor_eventsource::helper::{create_subscription, get_subscription};
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::{Event, Stream};
use redis::{Client as RedisClient, Commands};
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
pub async fn list(_admin: Admin, redis: &State<RedisClient>) -> Template {
    let mut connection = redis.get_connection().expect("cannot connect to redis");

    let raw_data: Option<String> = connection
        .get(APPLICATION_LIST_REDIS_KEY)
        .expect("cannot get data");

    let application_list = match raw_data {
        None => Vec::new(),
        Some(list) => list.split("|").map(|s| s.to_string()).collect(),
    };

    Template::render(
        "admin/applications",
        context! {
            applications: application_list
        },
    )
}

#[get("/admin/application/<id>")]
pub async fn get(_admin: Admin, id: &str) -> Template {
    Template::render("admin/application_form", context! {})
}

#[post("/admin/update-application/<id>", data = "<application>")]
pub async fn update(
    _admin: Admin,
    application: Form<Application<'_>>,
    repository: &State<ApplicationRepository>,
    id: &str,
) -> Template {
    let key = ModelKey::new(AUTH_APPLICATION_STREAM, id.parse().unwrap());
    todo!("todo update");

    Template::render("admin/index", context! {})
}

#[get("/admin/new-application")]
pub async fn get_create(_admin: Admin) -> Template {
    Template::render("admin/application_form", context! {})
}

#[post("/admin/new-application", data = "<application>")]
pub async fn post_create(
    _admin: Admin,
    application: Form<Application<'_>>,
    repository: &State<ApplicationRepository>,
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
