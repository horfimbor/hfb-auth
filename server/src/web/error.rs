use crate::session::get_session;
use anyhow::{anyhow, Error};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};

pub fn get_routes() -> Vec<Route> {
    routes![error,]
}

#[derive(Responder)]
#[response(status = 500, content_type = "html")]
pub struct ErrorPage {
    error: Template,
}


impl ErrorPage {
    #[cfg(debug_assertions)]
    pub fn new(error: Error, file: &str, line: u32) -> Self {
        Self {
            error: Template::render(
                "500",
                context! {
                    error: error.to_string(),
                    file: file,
                    line: line
                },
            ),
        }
    }
    #[cfg(not(debug_assertions))]
    pub fn new(error: Error, file: &str, line: u32) -> Self {
        
        println!("{file}:{line} : {error}");
        
        Self {
            error: Template::render(
                "500",
                context! {
                    error: error.to_string(),
                },
            ),
        }
    }
}

#[macro_export]
macro_rules! anyhow_error {
    ($e:tt) => {
        ErrorPage::new($e, file!(), line!())
    };
}
#[macro_export]
macro_rules! other_error {
    ($e:tt) => {
        ErrorPage::new(anyhow!($e), file!(), line!())
    };
}

#[get("/error")]
async fn error() -> Result<(), ErrorPage> {
    let e = anyhow!("So you want an error 🤔");
    Err(anyhow_error!(e))
}
