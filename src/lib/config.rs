use std::fmt::{Display, Formatter};
use std::io::{Error as IOError, ErrorKind};
use std::path::Path;

use figment::{
    Error as FigmentError,
    Figment,
    map,
    Metadata,
    Profile,
    Provider,
    providers::{Env, Format, Serialized, Toml},
    value::{Dict, Map},
};
use serde::{Deserialize, Serialize};

use super::result::Error;
use super::cli::Cli;

/* -------------------------------------- Util functions --------------------------------------- */

pub static DEFAULT_CONF_PATH: &str = "/etc/polar/polar.toml";

pub fn with_db_pool(figment: Figment) -> Result<Figment, FigmentError> {
    figment
        .extract()
        .map(|config: Config| map!["postgresql_pool" => map!["url" => config.database.to_string()]])
        .map(|pool| Figment::from(("databases", pool)))
        .map(|db_figment| figment.merge(db_figment))
}

/* --------------------------------------- File handling --------------------------------------- */

fn from_file(file_path: Option<&str>) -> Result<Figment, IOError> {
    let was_specified = file_path.is_some();
    let path_str = file_path.unwrap_or(DEFAULT_CONF_PATH);
    let path = Path::new(path_str);

    match path.exists() {
        false if was_specified => Err(IOError::new(ErrorKind::NotFound, path_str)),
        false => Ok(Figment::new()),
        true if !path.is_file() => Err(IOError::new(
            ErrorKind::Other,
            format!("{}, is a directory", path_str),
        )),
        true => Ok(Figment::from(Toml::file(path).nested())),
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

/// General Polar startup and runtime configuration store, attached to rocket as
/// a managed state for consumption.
///
/// # Overview
///
/// The `Config` structure is loaded at application startup by reading through 4
/// different sources. Each subsequent source overrides previously obtained
/// values if a conflict occurs. The providers of configuration values are, in
/// order, as follow:
///
/// 1. __Defaults__: A set of default values that may, or may not, work
/// depending on your environment.
/// 2. __Configuration file__: A TOML-formatted file. The path of this file
/// defaults to _"/etc/polar/polar.toml"_ and may be provided as an argument
/// during startup.
/// 3. __Environment variables__: Any environment variable prefixed with
/// _"POLAR\_"_ (_e.g_ __POLAR_PORT__) will be read as a candidate for
/// configuration.
/// 4. __Program arguments__: Any user provided arguments at application
/// startup will be parsed as an [Args] structure which will subsequently see
/// a subset of its data converted to configuration values.
///
/// # Example
///
/// As an example, consider and endpoint that would consume the configured jwt
/// lifetime in and endpoint. The token lifetime was set to 900 seconds in the
/// configuration file but was overridden to be 600 seconds when the app was
/// started.
///
/// ```rust,no_run,compile_fail
/// use rocket::State;
/// use polar::config::Config;
///
/// #[get("/lifetime")]
/// fn lifetime(config: State<Config>) -> u16 {
///     config.security.jwt_lifetime
/// }
/// ```
///
/// Said endpoint would return the number 600 to any consumer.
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

    pub fn figment(cli: Cli) -> Result<Figment, Error<'a>> {
        let base = Figment::from(rocket::Config::default());

        let profile = cli
            .profile()
            .unwrap_or(Profile::from_env_or("POLAR_PROFILE", "default"));

        let default_config = Figment::from(Serialized::defaults(Config::default()));
        let file_config = from_file(cli.configuration.as_ref().map(String::as_str))?;
        let env_config = Figment::from(Env::prefixed("POLAR_"));
        let args_config = Figment::from(cli);

        let config = base
            .merge(default_config)
            .merge(file_config)
            .merge(env_config)
            .merge(args_config)
            .select(profile.as_str());

        Ok(with_db_pool(config)?.select(profile.as_str()))
    }
}

/* ------------------------------------------- Tests ------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::{
        Config,
        super::cli::{Cli, Command, Serve},
    };
    use crate::lib::config::from_file;
    use figment::{Error as FigmentError, Figment, Jail, Profile};

    fn cli(
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
    ) -> Cli {
        Cli {
            configuration,
            profile,
            command: Command::Serve(Serve {
                address,
                port,
                database_host,
                database_port,
                database_user,
                database_password,
                database_schema,
                jwt_secret,
                jwt_lifetime,
            })
        }
    }

    fn empty_cli() -> Cli {
        cli(None, None, None, None, None, None, None, None, None, None, None, )
    }

    // Arguments tests

    #[test]
    fn empty_arguments() {
        let args = Figment::from(empty_cli());

        let figment = Figment::from(Config::default()).merge(args);
        let config: Result<Config, FigmentError> = figment.extract();

        match config {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "{}", e),
        }
    }

    #[test]
    fn args_default_profile() {
        let args = Figment::from(cli(
            None,
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
        let args = Figment::from(cli(
            None,
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
        let args = Figment::from(cli(
            None,
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

        assert_eq!(config.address, "192.168.1.42");
        assert_eq!(config.port, 4200);
        assert_eq!(config.database.host, "42.42.42.42");
        assert_eq!(config.database.port, 4242);
        assert_eq!(config.database.user, "test");
        assert_eq!(config.database.password, "test");
        assert_eq!(config.database.schema, "test");
        assert_eq!(config.security.jwt_secret, "secret");
        assert_eq!(config.security.jwt_lifetime, 42);
    }

    #[test]
    fn args_random_values_missing() {
        let args = Figment::from(cli(
            None,
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

        assert_eq!(config.address, "192.168.1.42");
        assert_eq!(config.port, 4200);
        assert_eq!(config.database.host, "42.42.42.42");
        assert_eq!(config.database.port, default_config.database.port);
        assert_eq!(config.database.user, "test");
        assert_eq!(config.database.password, default_config.database.password);
        assert_eq!(config.database.schema, default_config.database.schema);
        assert_eq!(config.security.jwt_secret, "secret");
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
                .merge(from_file(Some("polar.toml")).unwrap())
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
                Figment::from(Config::default()).merge(from_file(Some("polar.toml")).unwrap());
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
            let args = cli(
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

            assert_eq!(config.address, "42.42.42.42");

            Ok(())
        })
    }

    #[test]
    fn args_specify_nonexistent_file() {
        let args = cli(
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

    // Env

    #[test]
    fn env_select_profile() {
        Jail::expect_with(|jail| {
            jail.set_env("POLAR_PROFILE", "custom");
            let config = Config::figment(empty_cli()).unwrap();
            assert_eq!(config.profile(), "custom");

            Ok(())
        })
    }

    // Precedence

    #[test]
    fn precedence_env_over_file() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "42.42.42.42"
                "#,
            )?;

            jail.set_env("POLAR_ADDRESS", "0.0.0.0");
            let figment = Config::figment(empty_cli()).unwrap();
            let config: Config = figment.extract()?;

            assert_eq!(config.address, "0.0.0.0");

            Ok(())
        })
    }

    #[test]
    fn precedence_args_over_file() {
        Jail::expect_with(|jail| {
            let args = cli(
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
    fn precedence_args_over_env() {
        Jail::expect_with(|jail| {
            let args = cli(
                None,
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

            jail.set_env("POLAR_ADDRESS", "0.0.0.0");

            let figment = Config::figment(args).unwrap();
            let config: Config = figment.extract()?;

            assert_eq!(config.address, "192.168.1.42");

            Ok(())
        })
    }

    #[test]
    fn precedence_args_over_env_over_file() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "1.1.1.1"
                port = 1111
                [default.database]
                host = "1.1.1.1"
                "#,
            )?;

            jail.set_env("POLAR_ADDRESS", "2.2.2.2");
            jail.set_env("POLAR_PORT", "2222");

            let args = cli(
                Some("polar.toml".to_string()),
                None,
                Some("3.3.3.3".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );
            let figment = Config::figment(args).unwrap();
            let config: Config = figment.select("default").extract()?;

            assert_eq!(config.address, "3.3.3.3"); // Arg over env over file
            assert_eq!(config.port, 2222); // env over file
            assert_eq!(config.database.host, "1.1.1.1"); // file

            Ok(())
        })
    }
}
