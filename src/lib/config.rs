use rocket::figment::Error as FigmentError;
use std::fmt::{Display, Formatter};
use std::io::{Error as IOError, ErrorKind};
use std::path::Path;

use clap::Parser;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    value::{Dict, Map, Value},
    Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};

use crate::lib::database;
use crate::lib::result::Error;

/* -------------------------------------- Util functions --------------------------------------- */

fn ref_str(s: &Option<String>) -> Option<&str> {
    s.as_ref().map(|s| s.as_str())
}

fn prepare_tuples(xs: Vec<(&str, Option<Value>)>) -> Vec<(String, Value)> {
    return xs
        .into_iter()
        .filter(|(_, v)| v.is_some())
        .map(|(k, v)| (k.to_string(), v.unwrap()))
        .collect();
}

fn defaults() -> Figment {
    Figment::from(Serialized::defaults(Config::default()))
}

fn from_env() -> Figment {
    Figment::from(Env::prefixed("POLAR_"))
}

/* --------------------------------------- File handling --------------------------------------- */

// TODO - Add Yaml and json parsing to config file options
enum ConfigFileType {
    AUTO,
    TOML,
    JSON,
    YAML,
}

fn from_file(config_file_path: &str) -> Result<Figment, IOError> {
    let path = Path::new(config_file_path);
    if path.exists() {
        if path.is_file() {
            Ok(Figment::from(Toml::file(path).nested()))
        } else {
            // TODO - Set to ErrorKind::IsADirectory (see https://github.com/rust-lang/rust/issues/86442)
            Err(IOError::new(
                ErrorKind::Other,
                format!("{}, is a directory", config_file_path),
            ))
        }
    } else {
        Err(IOError::new(ErrorKind::NotFound, config_file_path))
    }
}

/* --------------------------------------- Args Parsing ---------------------------------------- */

fn from_args(args: Args) -> Figment {
    Figment::from(args)
}

#[derive(Parser, Debug)]
#[clap(
    author = "findot",
    version = "0.0.1",
    about = "TODO",
    long_about = "TODO"
)]
pub struct Args {
    /// The configuration file path
    #[clap(short, long, default_value = "/etc/polar/polar.toml")]
    configuration: String,

    /// The configuration profile to use
    #[clap(long)]
    profile: Option<String>,

    /// The interface on which polar should listen
    #[clap(short, long)]
    address: Option<String>,

    /// The port on which polar should listen
    #[clap(short, long)]
    port: Option<u16>,

    /// The server IP address on which the database must be contacted
    #[clap(long)]
    database_host: Option<String>,

    /// The server port on which the database must be contacted
    #[clap(long)]
    database_port: Option<u16>,

    /// The user with which polar will authenticate to the database
    #[clap(long)]
    database_user: Option<String>,

    /// The password with which polar will authenticate to the database
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

    fn profile(&self) -> Option<Profile> {
        self.profile.as_ref().map(|p| Profile::new(p.as_str()))
    }
}

impl Args {
    fn new(
        configuration: Option<String>,
        profile: Option<String>,
        address: Option<String>,
        port: Option<u16>,
        database_host: Option<String>,
        database_port: Option<u16>,
        database_user: Option<String>,
        database_password: Option<String>,
        database_schema: Option<String>,
        jwt_secret: Option<String>,
        jwt_lifetime: Option<u16>,
    ) -> Self {
        Args {
            configuration: configuration.unwrap_or("/etc/polar/polar.toml".to_string()),
            profile,
            address,
            port,
            database_host,
            database_port,
            database_user,
            database_password,
            database_schema,
            jwt_secret,
            jwt_lifetime,
        }
    }
}

/* -------------------------------------- Database Config -------------------------------------- */

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
            user: "polar".to_string(),
            password: "polar".to_string(),
            schema: "polar".to_string(),
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Library config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, FigmentError> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}

impl<'a> Config {
    pub fn from<T: Provider>(provider: T) -> Result<Config, FigmentError> {
        Figment::from(provider).extract()
    }

    pub fn figment(args: Args) -> Result<Figment, Error<'a>> {
        let base = Figment::from(rocket::Config::default());

        let profile = args
            .profile
            .as_ref()
            .map(|p| Profile::from(p))
            .unwrap_or(Profile::from_env_or("POLAR_PROFILE", "default"));

        let default_config = defaults();
        let file_config = from_file(args.configuration.as_str())?;
        let env_config = from_env();
        let args_config = from_args(args);

        let config = base
            .merge(default_config)
            .merge(file_config)
            .merge(env_config)
            .merge(args_config)
            .select(profile);

        Ok(database::with_pool(config)?)
    }
}

/* ------------------------------------------- Tests ------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::{Args, Config};
    use crate::lib::config::from_file;
    use figment::{Error as FigmentError, Figment, Jail, Profile};

    // Arguments tests

    #[test]
    fn empty_arguments() {
        let args = Figment::from(Args::new(
            Some("/etc/polar/polar.toml".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        let figment = Figment::from(Config::default()).merge(args);
        let config: Result<Config, FigmentError> = figment.extract();

        match config {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "{}", e),
        }
    }

    #[test]
    fn args_default_profile() {
        let args = Figment::from(Args::new(
            Some("/etc/polar/polar.toml".to_string()),
            Some("default".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        let figment = Figment::from(Config::default()).merge(args);

        assert_eq!(figment.profile(), &Profile::default())
    }

    #[test]
    fn args_custom_profile() {
        let args = Figment::from(Args::new(
            Some("/etc/polar/polar.toml".to_string()),
            Some("custom".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        let figment = Figment::from(Config::default()).merge(args);

        assert_eq!(figment.profile(), &Profile::new("custom"))
    }

    #[test]
    fn args_random_values() {
        let args = Figment::from(Args::new(
            Some("/etc/polar/polar.toml".to_string()),
            None,
            Some("192.168.1.42".to_string()),
            Some(4200),
            Some("42.42.42.42".to_string()),
            Some(4242),
            Some("test".to_string()),
            Some("test".to_string()),
            Some("test".to_string()),
            Some("secret".to_string()),
            Some(42),
        ));

        let figment = Figment::from(Config::default()).merge(args);
        let config_result: Result<Config, FigmentError> = figment.extract();

        match &config_result {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "{}", e),
        };

        let config = config_result.unwrap();

        assert_eq!(config.address.as_str(), "192.168.1.42");
        assert_eq!(config.port, 4200);
        assert_eq!(config.database.host.as_str(), "42.42.42.42");
        assert_eq!(config.database.port, 4242);
        assert_eq!(config.database.user.as_str(), "test");
        assert_eq!(config.database.password.as_str(), "test");
        assert_eq!(config.database.schema.as_str(), "test");
        assert_eq!(config.security.jwt_secret.as_str(), "secret");
        assert_eq!(config.security.jwt_lifetime, 42);
    }

    #[test]
    fn args_random_values_missing() {
        let args = Figment::from(Args::new(
            Some("/etc/polar/polar.toml".to_string()),
            None,
            Some("192.168.1.42".to_string()),
            Some(4200),
            Some("42.42.42.42".to_string()),
            None,
            Some("test".to_string()),
            None,
            None,
            Some("secret".to_string()),
            Some(42),
        ));

        let figment = Figment::from(Config::default()).merge(args);
        let config_result: Result<Config, FigmentError> = figment.extract();

        match &config_result {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "{}", e),
        };

        let config = config_result.unwrap();
        let default_config = Config::default();

        assert_eq!(config.address.as_str(), "192.168.1.42");
        assert_eq!(config.port, 4200);
        assert_eq!(config.database.host.as_str(), "42.42.42.42");
        assert_eq!(config.database.port, default_config.database.port);
        assert_eq!(config.database.user.as_str(), "test");
        assert_eq!(config.database.password, default_config.database.password);
        assert_eq!(config.database.schema, default_config.database.schema);
        assert_eq!(config.security.jwt_secret.as_str(), "secret");
        assert_eq!(config.security.jwt_lifetime, 42);
    }

    // Config file tests

    #[test]
    fn file_empty() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "polar.toml",
                r#"
                [default]
                "#,
            )?;

            let default_config = Config::default();
            let file_config: Config = Figment::from(&default_config)
                .merge(from_file("polar.toml").unwrap())
                .extract()?;

            assert_eq!(default_config, file_config);

            Ok(())
        })
    }

    #[test]
    fn file_different_profiles() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "127.0.0.1"
                [custom]
                address = "0.0.0.0"
                "#,
            )?;

            let file_config =
                Figment::from(Config::default()).merge(from_file("polar.toml").unwrap());
            let default_config: Config =
                file_config.clone().select(Profile::default()).extract()?;
            let custom_config: Config = file_config
                .clone()
                .select(Profile::new("custom"))
                .extract()?;

            assert_ne!(default_config.address, custom_config.address);

            Ok(())
        })
    }

    // Args and file

    #[test]
    fn args_select_config_file() {
        Jail::expect_with(|jail| {
            let args = Args::new(
                Some("polar.toml".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );

            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "42.42.42.42"
            "#,
            )?;

            let config_result = Config::figment(args);
            match &config_result {
                Ok(_) => assert!(true),
                Err(e) => assert!(false, "{}", e),
            }
            let config: Config = config_result.unwrap().extract()?;

            assert_eq!(config.address.as_str(), "42.42.42.42");

            Ok(())
        })
    }

    #[test]
    fn args_precedence_over_file() {
        Jail::expect_with(|jail| {
            let args = Args::new(
                Some("polar.toml".to_string()),
                None,
                Some("192.168.1.42".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );

            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "42.42.42.42"
                "#,
            )?;

            let config_result = Config::figment(args);
            match &config_result {
                Ok(_) => assert!(true),
                Err(e) => assert!(false, "{}", e),
            }
            let config: Config = config_result.unwrap().extract()?;

            assert_eq!(config.address.as_str(), "192.168.1.42");

            Ok(())
        })
    }

    #[test]
    fn args_specify_nonexistent_file() {
        let args = Args::new(
            Some("i_definitely_dont_exist.toml".to_string()),
            None,
            Some("192.168.1.42".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(Config::figment(args).is_err())
    }
}
