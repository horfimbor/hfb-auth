use crate::model::MariadDb;
use rocket::form::Form;
use rocket::http::CookieJar;
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

#[post("/login", data = "<login>")]
async fn login(
    login: Form<Login<'_>>,
    _maria_db: &State<MariadDb>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    cookies.add_private(("pseudo", login.pseudo.to_string()));

    Redirect::to(uri!(account()))
}

#[get("/account")]
async fn account(cookies: &CookieJar<'_>, _maria_db: &State<MariadDb>) -> Template {
    let data = cookies
        .get_private("pseudo")
        .map_or("no cookie".to_string(), |crumb| crumb.value().to_string());

    Template::render(
        "account",
        context! {
            name: data
        },
    )
}
