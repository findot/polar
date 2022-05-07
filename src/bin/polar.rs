extern crate core;

use clap::Parser;

use polar::{app::App, cli::Cli, result::Result};
use rocket;

#[rocket::main]
async fn main() -> Result<'static, ()>{
    let args: Cli = Cli::parse();
    let app = App::new(args)?;
    app.run().await;
    Ok(())
}
