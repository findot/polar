use std::process::exit;

use crate::cli::{Cli, Command, DumpFormat};
use crate::config::Config;
use crate::database::{migrate as db_migrate, Db};
use crate::result::Result;
use argon2::Argon2;
use figment::Figment;
use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

pub mod core;
pub mod routes;

pub struct App {
    args: Cli,
    config: Config,
    figment: Figment,
}

impl App {
    pub async fn new<'a>(args: Cli) -> Result<'a, Self> {
        let figment = Config::figment(&args).await?;
        let config: Config = figment.extract()?;
        Ok(Self {
            args,
            figment,
            config,
        })
    }

    pub async fn serve<'a>(&self) -> Result<'a, ()> {
        rocket::custom(&self.figment)
            .attach(AdHoc::config::<Config>())
            .attach(Db::init())
            .manage(Argon2::default())
            .mount("/", routes::collect())
            .launch()
            .await?;
        Ok(())
    }

    pub fn migrate<'a>(&self) -> Result<'a, ()> {
        db_migrate(&self.config.database)?;
        Ok(())
    }

    pub fn show<'a>(&self, format: Option<DumpFormat>) -> Result<'a, ()> {
        let fmt = format.unwrap_or(DumpFormat::Json);
        println!("{}", fmt.to_string(&self.config)?);
        Ok(())
    }

    pub async fn run(&self) {
        if let Err(e) = match &self.args.command {
            Command::Serve(_) => self.serve().await,
            Command::Migrate(_) => self.migrate(),
            Command::Show(show) => self.show(show.format),
        } {
            eprintln!("Error: {}", e.to_string());
            exit(1);
        }
    }
}
