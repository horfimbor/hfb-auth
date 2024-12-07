use crate::user::User;
use hfb_auth_shared::user::AuthUserState;
use rocket::http::CookieJar;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Route};
use rocket_dyn_templates::{context, Template};

pub fn get_account_routes() -> Vec<Route> {
    routes![index]
}

#[get("/account")]
pub async fn index(user: User) -> Template {
    Template::render(
        "account",
        context! {
            name: "bob"
        },
    )
}
