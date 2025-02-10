use crate::constants::{APPLICATION_LIST_REDIS_KEY, AUTH_APPLICATION_UUID};
use crate::user::Admin;
use crate::web::error::ErrorPage;
use crate::{anyhow_error, other_error, ApplicationRepository};
use anyhow::anyhow;
use anyhow::Context;
use eventstore::Client;
use hfb_auth_shared::application::{ApplicationCommand, ApplicationList, PrivateApplicationEvent};
use hfb_auth_shared::AUTH_APPLICATION_STREAM;
use horfimbor_eventsource::helper::{create_subscription, get_subscription};
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::{Event, Stream};
use redis::{Client as RedisClient, Commands};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use url::{Host, Url};
use uuid::{Error, Uuid};

#[derive(FromForm, Debug)]
pub struct Application<'r> {
    name: &'r str,
    host: &'r str,
}

#[get("/admin/applications")]
pub async fn list(_admin: Admin, redis: &State<RedisClient>) -> Result<Template, ErrorPage> {
    let mut connection = redis.get_connection().expect("cannot connect to redis");

    let raw_data: Option<String> = connection
        .get(APPLICATION_LIST_REDIS_KEY)
        .expect("cannot get data");

    let application_list: Vec<ApplicationList> = match raw_data {
        None => Vec::new(),
        Some(list) => serde_json::from_str(&list)
            .context("cannot deserialize application list in redis")
            .map_err(|a| anyhow_error!(a))?,
    };

    Ok(Template::render(
        "admin/applications",
        context! {
            applications: application_list
        },
    ))
}

#[get("/admin/application/<id>")]
pub async fn get(_admin: Admin, id: &str) -> Template {
    Template::render("admin/application_update", context! {})
}

#[post("/admin/update-application/<id>", data = "<application>")]
pub async fn update(
    _admin: Admin,
    application: Form<Application<'_>>,
    repository: &State<ApplicationRepository>,
    id: &str,
) -> Result<Template, ErrorPage> {
    let uuid = match id.parse::<Uuid>() {
        Ok(u) => u,
        Err(e) => return Err(other_error!(e)),
    };
    let key = ModelKey::new(AUTH_APPLICATION_STREAM, uuid);
    todo!("todo update");

    Ok(Template::render("admin/index", context! {}))
}

#[get("/admin/new-application")]
pub async fn get_create(_admin: Admin) -> Template {
    Template::render("admin/application_create", context! {})
}

#[post("/admin/new-application", data = "<application>")]
pub async fn post_create(
    _admin: Admin,
    application: Form<Application<'_>>,
    repository: &State<ApplicationRepository>,
) -> Result<Template, ErrorPage> {
    let key = ModelKey::new_uuid_v8(
        AUTH_APPLICATION_STREAM,
        AUTH_APPLICATION_UUID,
        application.host,
    );

    let model = repository
        .add_command(
            &key,
            ApplicationCommand::Create {
                name: application.name.to_string(),
                host: Url::parse(application.host).map_err(|e| other_error!(e))?,
            },
            None,
        )
        .await
        .map_err(|e| other_error!(e))?;

    Ok(Template::render("admin/index", context! {}))
}
