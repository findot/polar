extern crate core;

use clap::Parser;
use std::process::exit;

use polar::app;
use polar::{
    cli::{Cli, Command, DumpFormat},
    config::Config,
    database::{migrate as db_migrate, DbConnection},
    result::Result,
};

use rocket;
use rocket::fairing::AdHoc;

async fn serve<'a>(args: &Cli) -> Result<'a, ()> {
    let config = Config::figment(args)?;
    let result = rocket::custom(config)
        .attach(AdHoc::config::<Config>())
        .attach(DbConnection::fairing())
        .mount("/", app::routes::collect())
        .launch()
        .await?;
    Ok(result)
}

fn migrate<'a>(args: &Cli) -> Result<'a, ()> {
    let figment = Config::figment(args)?;
    let config: Config = figment.extract()?;
    db_migrate(&config.database, true)?;
    Ok(())
}

fn show_conf<'a>(args: &Cli, format: Option<DumpFormat>) -> Result<'a, ()> {
    let fmt = format.unwrap_or(DumpFormat::Json);
    let figment = Config::figment(args)?;
    let config: Config = figment.extract()?;
    println!("{}", fmt.to_string(&config)?);
    Ok(())
}

#[rocket::main]
async fn main() {
    let args: Cli = Cli::parse();

    if let Err(e) = match &args.command {
        Command::Serve(_) => serve(&args).await,
        Command::Migrate(_) => migrate(&args),
        Command::Show(show) => show_conf(&args, show.format),
    } {
        eprintln!("Error: {}", e.to_string());
        exit(1);
    }
}
