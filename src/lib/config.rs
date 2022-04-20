use rocket::figment::Error as FigmentError;
use std::fmt::{Display, Formatter};
use std::path::Path;

use clap::Parser;
use rocket::{
    figment::value::{Dict, Map, Value},
    figment::{
        providers::{Env, Format, Serialized, Toml},
        Figment, Metadata, Profile, Provider,
    },
};
// use rocket::form::error::Entity::Value;
use serde::{Deserialize, Serialize};

use crate::lib::database;
use crate::lib::result::Error;

/* -------------------------------------- Util functions --------------------------------------- */

fn defaults() -> Figment {
    Figment::from(Serialized::defaults(Config::default()))
}

// TODO - Deal with provided but nonexistent configuration file
fn from_file(config_file_path: &str) -> Figment {
    let path = Path::new(config_file_path);
    if path.exists() {
        Figment::from(Toml::file(path).nested())
    } else {
        Figment::new()
    }
}

fn from_env() -> Figment {
    Figment::from(Env::prefixed("LANCE_"))
}

fn from_args(args: Args) -> Figment {
    Figment::from(args)
}

/* --------------------------------------- Args Parsing ---------------------------------------- */

#[derive(Parser, Debug)]
#[clap(
    author = "findot",
    version = "0.0.1",
    about = "TODO",
    long_about = "TODO"
)]
pub struct Args {
    /// The configuration file path
    #[clap(short, long, default_value = "/etc/lance/lance.toml")]
    configuration: String,

    /// The configuration profile to use
    #[clap(long)]
    profile: Option<String>,

    /// The interface on which lance should listen
    #[clap(short, long)]
    address: Option<String>,

    /// The port on which lance should listen
    #[clap(short, long)]
    port: Option<u16>,

    /// The server IP address on which the database must be contacted
    #[clap(long)]
    database_host: Option<String>,

    /// The server port on which the database must be contacted
    #[clap(long)]
    database_port: Option<u16>,

    /// The user with which lance will authenticate to the database
    #[clap(long)]
    database_user: Option<String>,

    /// The password with which lance will authenticate to the database
    #[clap(long)]
    database_password: Option<String>,

    /// The database schema to use
    #[clap(long)]
    database_schema: Option<String>,

    /// The seed to use for jwt generation
    #[clap(long)]
    jwt_secret: Option<String>,

    /// The lifespan (in seconds) during which any emitted jwt token will be valid
    #[clap(long)]
    jwt_lifetime: Option<u16>,
}

fn prepare_tuples(xs: Vec<(&str, Option<Value>)>) -> Vec<(String, Value)> {
    return xs
        .into_iter()
        .filter(|(_, v)| v.is_some())
        .map(|(k, v)| (k.to_string(), v.unwrap()))
        .collect();
}

fn ref_str(s: &Option<String>) -> Option<&str> {
    s.as_ref().map(|s| s.as_str())
}

impl Provider for Args {
    fn metadata(&self) -> Metadata {
        Metadata::named("Arguments")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, FigmentError> {
        let database = Dict::from_iter(prepare_tuples(vec![
            ("host", ref_str(&self.database_host).map(Value::from)),
            ("port", self.database_port.map(Value::from)),
            ("user", ref_str(&self.database_user).map(Value::from)),
            (
                "password",
                ref_str(&self.database_password).map(Value::from),
            ),
            ("schema", ref_str(&self.database_schema).map(Value::from)),
        ]));

        let security = Dict::from_iter(prepare_tuples(vec![
            ("jwt_secret", ref_str(&self.jwt_secret).map(Value::from)),
            ("jwt_lifetime", self.jwt_lifetime.map(Value::from)),
        ]));

        let root = Dict::from_iter(prepare_tuples(vec![
            ("address", ref_str(&self.address).map(Value::from)),
            ("port", self.port.map(Value::from)),
            ("database", Some(Value::from(database))),
            ("security", Some(Value::from(security))),
        ]));

        let profile_str = ref_str(&self.profile).unwrap_or("default");
        let profile = Profile::from(profile_str);
        Ok(Map::from_iter(vec![(profile, root)]))
    }
}

/* -------------------------------------- Database Config -------------------------------------- */

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub schema: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "127.0.0.1".to_string(),
            port: 5432,
            user: "lance".to_string(),
            password: "lance".to_string(),
            schema: "lance".to_string(),
        }
    }
}

impl Display for DatabaseConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.schema
        )
    }
}

/* -------------------------------------- Security Config -------------------------------------- */

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_lifetime: u16,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            jwt_secret: "secret".to_string(),
            jwt_lifetime: 900,
        }
    }
}

/* -------------------------------------- General Config --------------------------------------- */

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub address: String,
    pub port: u16,

    pub security: SecurityConfig,
    pub database: DatabaseConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: "127.0.0.1".to_string(),
            port: 8080,

            security: SecurityConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

impl<'a> Config {
    pub fn from<T: Provider>(provider: T) -> Result<Config, FigmentError> {
        Figment::from(provider).extract()
    }

    pub fn figment(args: Args) -> Result<Figment, Error<'a>> {
        let base = Figment::from(rocket::Config::default());

        let default_config = defaults();
        let file_config = from_file(args.configuration.as_str());
        let env_config = from_env();
        let args_config = from_args(args);

        // TODO - Deal with provided profile
        let profile = Profile::from_env_or("LANCE_PROFILE", "default");

        let config = base
            .merge(default_config)
            .merge(file_config)
            .merge(env_config)
            .merge(args_config)
            .select(profile);

        Ok(database::with_pool(config)?)
    }
}
