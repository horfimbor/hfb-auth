use crate::controllers::{connexion, COOKIE_SESSION};
use crate::model::MariadDb;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};

#[get("/authorize?<redirect>")]
pub async fn authorize(
    cookies: &CookieJar<'_>,
    maria_db: &State<MariadDb>,
    redirect: &str,
) -> Result<Template, Status> {
    dbg!(&redirect);

    let application = maria_db
        .get_application_by_host(redirect)
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?
        .ok_or(Status::NotFound)?;

    let data = cookies.get(COOKIE_SESSION);
    match data {
        None => Ok(connexion::render_login(redirect, None)),
        Some(_) => Ok(Template::render(
            "authorize",
            context! {
                application_name: application.name(),
                redirect: redirect
            },
        )),
    }
}

#[derive(FromForm, Debug)]
struct Authorize<'r> {
    redirect: &'r str,
}

#[post("/authorize", data = "<authorize>")]
pub async fn authorize_form(
    cookies: &CookieJar<'_>,
    maria_db: &State<MariadDb>,
    authorize: Form<Authorize<'_>>,
) -> Result<Redirect, Status> {
    let application = maria_db
        .get_application_by_host(authorize.redirect)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let data = cookies.get(COOKIE_SESSION).ok_or(Status::NotFound)?;

    let account = maria_db
        .get_user_by_id(data.value())
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let id = maria_db
        .new_one_time_token(&application, &account)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to(format!(
        "{}/auth?token={id}",
        authorize.redirect
    )))
}
