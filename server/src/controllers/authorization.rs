use crate::controllers::url_parsing::RedirectUrl;
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
    redirect: RedirectUrl,
) -> Result<Template, Status> {
    dbg!(&redirect);

    let application = maria_db
        .get_application_by_host(redirect.host())
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?
        .ok_or(Status::NotFound)?;

    let data = cookies.get(COOKIE_SESSION);
    match data {
        None => Ok(connexion::render_login(Some(redirect.url()), None)),
        Some(_) => Ok(Template::render(
            "authorize",
            context! {
                application_name: application.name(),
                redirect: redirect.url().as_str()
            },
        )),
    }
}

#[derive(FromForm, Debug)]
struct Authorize {
    redirect: RedirectUrl,
}

#[post("/authorize", data = "<authorize>")]
pub async fn authorize_form(
    cookies: &CookieJar<'_>,
    maria_db: &State<MariadDb>,
    authorize: Form<Authorize>,
) -> Result<Redirect, Status> {
    todo!();
    // let application = maria_db
    //     .get_application_by_host(authorize.clone().redirect.host)
    //     .await
    //     .map_err(|_| Status::InternalServerError)?
    //     .ok_or(Status::NotFound)?;
    //
    // let data = cookies.get(COOKIE_SESSION).ok_or(Status::NotFound)?;
    //
    // let account = maria_db
    //     .get_user_by_id(data.value())
    //     .await
    //     .map_err(|_| Status::InternalServerError)?
    //     .ok_or(Status::NotFound)?;
    //
    // let id = maria_db
    //     .new_one_time_token(&application, &account)
    //     .await
    //     .map_err(|_| Status::InternalServerError)?;
    //
    // Ok(Redirect::to(format!(
    //     "{}/auth?token={id}",
    //     authorize.redirect.url
    // )))
}
