extern crate core;

use polar::app;
use polar::lib::{config::Config, cli::Cli, cli::Command};
use polar::lib::database::DbConnection;

use clap::Parser;
use rocket::fairing::AdHoc;
use rocket::{launch, Build, Rocket};

#[launch]
fn rocket() -> Rocket<Build> {
    let args: Cli = Cli::parse();

    match &args.command {
        Command::Serve(_) => {
            let config = Config::figment(args).unwrap(); // TODO - Handle error
            rocket::custom(config)
                .attach(AdHoc::config::<Config>())
                .attach(DbConnection::fairing())
                .mount("/", app::routes::collect())
        },
        Command::Migrate(_) => panic!("Not implemented")
    }
}
