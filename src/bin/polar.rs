extern crate core;

use clap::Parser;

use polar::{cli::Cli, result::Result, App};

#[rocket::main]
async fn main() -> Result<'static, ()> {
    App::new(Cli::parse())?.run().await;
    Ok(())
}
