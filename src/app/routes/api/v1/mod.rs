use rocket::Route;
mod auth;

pub fn collect() -> Vec<Route> {
    routes!(auth::login, auth::register)
}
