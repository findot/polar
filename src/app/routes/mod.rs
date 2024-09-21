//! # Routes
//!
//! Group all routes of the application, classified by module, respecting the
//! route path.

mod api;

use rocket::Route;

pub fn collect() -> Vec<Route> {
    api::collect()
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
