#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel_migrations;

pub mod app;
mod lib;

pub use lib::api;
pub use lib::cli;
pub use lib::config;
pub use lib::database;
pub use lib::result;
