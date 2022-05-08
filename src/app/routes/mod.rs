//! # Routes
//!
//! Group all routes of the application, classified by module, respecting the
//! route path.

pub mod v1;

use rocket::Route;

pub fn collect() -> Vec<Route> {
    routes!(index)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
