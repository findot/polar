extern crate core;

use clap::Parser;

use polar::{app::App, cli::Cli, result::Result};
use rocket;

#[rocket::main]
async fn main() -> Result<'static, ()>{
    App::new(Cli::parse())?.run().await;
    Ok(())
}
