use rocket::Route;
mod v1;

pub fn collect() -> Vec<Route> {
    v1::collect()
}
