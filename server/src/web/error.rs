use crate::session::get_session;
use anyhow::{anyhow, bail, Context, Error};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};

pub fn get_routes() -> Vec<Route> {
    routes![error, error_anyhow]
}

#[derive(Responder)]
#[response(status = 500, content_type = "html")]
pub struct ErrorPage {
    error: Template,
}

#[derive(Responder)]
#[response(status = 500, content_type = "text")]
pub struct ErrorApi {
    error: String,
}

impl ErrorPage {
    #[cfg(debug_assertions)]
    pub fn new(error: Error, file: &str, line: u32) -> Self {
        println!("{file}:{line} : {error}");
        dbg!(&error);
        Self {
            error: Template::render(
                "500",
                context! {
                    error: format!("{error:?}"),
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
                    file: "",
                    line: ""
                },
            ),
        }
    }
}

impl ErrorApi {
    #[cfg(debug_assertions)]
    pub fn new(error: Error, file: &str, line: u32) -> Self {
        println!("{file}:{line} : {error}");
        Self {
            error: format!("{file}:{line} : {error}"),
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(error: Error, file: &str, line: u32) -> Self {
        println!("{file}:{line} : {error}");

        Self {
            error: error.to_string(),
        }
    }
}

#[macro_export]
macro_rules! anyhow_error_page {
    ($e:tt) => {
        ErrorPage::new($e, file!(), line!())
    };
}
#[macro_export]
macro_rules! other_error_page {
    ($e:tt) => {
        ErrorPage::new(anyhow!($e), file!(), line!())
    };
}

#[macro_export]
macro_rules! anyhow_error_api {
    ($e:tt) => {
        ErrorApi::new($e, file!(), line!())
    };
}
#[macro_export]
macro_rules! other_error_api {
    ($e:tt) => {
        ErrorApi::new(anyhow!($e), file!(), line!())
    };
}

#[get("/error")]
async fn error() -> Result<(), ErrorPage> {
    let e = anyhow!("So you want an error 🤔");
    Err(anyhow_error_page!(e))
}

#[get("/error_anyhow")]
async fn error_anyhow() -> Result<(), ErrorPage> {
    some_error_func().context("wrong in some error func").map_err(|e| anyhow_error_page!(e))?;
    let e = anyhow!("So you want an error 🤔");
    Err(anyhow_error_page!(e))
}

fn some_error_func() -> anyhow::Result<()>{
    bail!("something wrong")
}