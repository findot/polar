use polar::app;
use polar::lib::config::{Args, Config};
use polar::lib::database::DbConnection;

use clap::Parser;
use rocket::fairing::AdHoc;
use rocket::{launch, Build, Rocket};

#[launch]
fn rocket() -> Rocket<Build> {
    let args: Args = Args::parse();
    let conf = Config::figment(args).unwrap(); // TODO - Handle error

    rocket::custom(conf)
        .attach(AdHoc::config::<Config>())
        .attach(DbConnection::fairing())
        .mount("/", app::routes::collect())
}
