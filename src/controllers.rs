use crate::model::MariadDb;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};

pub fn get_routes() -> Vec<Route> {
    routes![index, login, account]
}

#[get("/")]
async fn index() -> Template {
    Template::render("index", context! {})
}

#[derive(FromForm, Debug)]
struct Login<'r> {
    pseudo: &'r str,
}

const COOKIE_NAME: &str = "RSESSID";

#[post("/login", data = "<login>")]
async fn login(
    login: Form<Login<'_>>,
    maria_db: &State<MariadDb>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    let user = maria_db.get_user(login.pseudo).await.map_err(|e| {
        dbg!(e);
        Status::InternalServerError
    })?;

    let user = match user {
        Some(u) => u,
        None => maria_db.create_user(login.pseudo).await.map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?,
    };

    cookies.add_private((COOKIE_NAME, user.uuid().to_string()));
    Ok(Redirect::to(uri!(account())))
}

#[get("/account")]
async fn account(cookies: &CookieJar<'_>, _maria_db: &State<MariadDb>) -> Template {
    let data = cookies
        .get_private(COOKIE_NAME)
        .map_or("no cookie".to_string(), |crumb| crumb.value().to_string());

    Template::render(
        "account",
        context! {
            name: data
        },
    )
}
