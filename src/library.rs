#[macro_use]
extern crate rocket;

pub mod app;
pub mod lib;

pub use lib::api;
pub use lib::config;
pub use lib::database;
pub use lib::result;
