use crate::model::MariadDb;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};

pub fn get_routes() -> Vec<Route> {
    routes![index, login, account, authorize]
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

    let mut cookie = Cookie::new(COOKIE_NAME,  user.uuid().to_string());
    cookie.set_same_site(Some(SameSite::Lax));

    cookies.add(cookie);
    Ok(Redirect::to(uri!(account())))
}

#[get("/account")]
async fn account(cookies: &CookieJar<'_>, _maria_db: &State<MariadDb>) -> Template {

    dbg!(cookies);

    let data = cookies
        .get(COOKIE_NAME)
        .map_or("no cookie".to_string(), |crumb| crumb.value().to_string());

    Template::render(
        "account",
        context! {
            name: data
        },
    )
}

#[get("/authorize?<redirect>")]
async fn authorize(cookies: &CookieJar<'_>, _maria_db: &State<MariadDb>, redirect: String) -> Template {

    dbg!(redirect);

    let data = cookies
        .get(COOKIE_NAME)
        .map_or("no cookie".to_string(), |crumb| crumb.value().to_string());

    Template::render(
        "account",
        context! {
            name: data
        },
    )
}
