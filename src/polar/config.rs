use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::io::{Error as IOError, ErrorKind};
use std::path::Path;

use figment::error::Kind;
use figment::{
    map,
    providers::{Env, Format, Serialized, Toml},
    value::{Dict, Map},
    Error as FigmentError, Figment, Metadata, Profile, Provider,
};
use rocket_db_pools::Database;
use serde::{Deserialize, Serialize};

use super::cli::Cli;
use super::crypto;
use super::database::Db;
use super::result::{CryptoError, Error};

/* -------------------------------------- Util functions --------------------------------------- */

pub static DEFAULT_CONF_PATH: &str = "/etc/polar/polar.toml";
static PRIV_KEY_CONF_PATH: &str = "security.private_key_path";
static PUB_KEY_CONF_PATH: &str = "security.public_key_path";

#[inline]
fn missing(key: &'static str) -> FigmentError {
    FigmentError::from(Kind::MissingField(Cow::Borrowed(key)))
}

#[inline]
fn incorrect<'a, 'b>(key: &'a str, reason: CryptoError) -> FigmentError {
    FigmentError::from(Kind::Message(format!(
        "Incorrect value for key '{}': {}",
        key, reason
    )))
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

impl DatabaseConfig {
    fn new(host: &str, port: u16, user: &str, password: &str, schema: &str) -> DatabaseConfig {
        Self {
            host: host.to_string(),
            port,
            user: user.to_string(),
            password: password.to_string(),
            schema: schema.to_string(),
        }
    }

    fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.schema
        )
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig::new("127.0.0.1", 5432, "polar", "polar", "polar")
    }
}

impl Display for DatabaseConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

/* -------------------------------------- Security Config -------------------------------------- */

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityConfig {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub jwt_lifetime: u16,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            private_key: vec![],
            public_key: vec![],
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
/// startup will be parsed as a [Cli] structure which will subsequently see
/// a subset of its data converted to configuration values.
///
/// # Example
///
/// As an example, consider and endpoint that would consume the configured jwt
/// lifetime. The token lifetime was set to 900 seconds in the configuration
/// file but was overridden to be 600 seconds when the app was started.
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

impl Config {
    pub fn from<T: Provider>(provider: T) -> Result<Config, FigmentError> {
        Figment::from(provider).extract()
    }

    pub async fn figment<'a>(cli: &Cli) -> Result<Figment, Error<'a>> {
        let base = Figment::from(rocket::Config::default());

        let profile = cli
            .profile()
            .unwrap_or(Profile::from_env_or("POLAR_PROFILE", "default"));

        let default_config = Figment::from(Serialized::defaults(Config::default()));
        let file_config = from_file(cli.configuration.as_ref().map(String::as_str))?;
        let env_config = Figment::from(Env::prefixed("POLAR_"));
        let cli_config = Figment::from(cli);

        let config = base
            .merge(default_config)
            .merge(file_config)
            .merge(env_config)
            .merge(cli_config)
            .select(profile.as_str());

        let db_config = Config::with_db_pool(config)?;
        let pk_config = Config::with_keys(db_config).await?;

        Ok(pk_config.select(profile.as_str()))
    }

    fn with_db_pool(figment: Figment) -> Result<Figment, FigmentError> {
        figment
            .extract()
            .map(|config: Config| map![Db::NAME => map!["url" => config.database.to_string()]])
            .map(|pool| Figment::from(("databases", pool)))
            .map(|db_figment| figment.merge(db_figment))
    }

    async fn with_keys(figment: Figment) -> Result<Figment, FigmentError> {
        let priv_p = figment.find_value(PRIV_KEY_CONF_PATH)?;
        let pub_p = figment.find_value(PUB_KEY_CONF_PATH)?;
        match (priv_p.as_str(), pub_p.as_str()) {
            (None, _) => Err(missing(PRIV_KEY_CONF_PATH)),
            (_, None) => Err(missing(PUB_KEY_CONF_PATH)),
            (Some(priv_path), Some(pub_path)) => {
                let priv_key = crypto::extract_key(priv_path)
                    .await
                    .map_err(|ce| incorrect(PRIV_KEY_CONF_PATH, ce))?;
                let pub_key = crypto::extract_key(pub_path)
                    .await
                    .map_err(|ce| incorrect(PUB_KEY_CONF_PATH, ce))?;
                let keys_figment =
                    Figment::from(("private_key", priv_key)).merge(("public_key", pub_key));
                Ok(figment.merge(keys_figment))
            }
        }
    }
}

/* ------------------------------------------- Tests ------------------------------------------- */

#[cfg(test)]
mod tests {
    use core::str;
    use std::future::Future;

    use super::{
        super::cli::{Cli, Command, Serve},
        Config,
    };
    use crate::polar::config::from_file;
    use figment::{Error as FigmentError, Figment, Jail, Profile};

    // Utils

    static PRIV_KEY_BYTES: &[u8] = include_bytes!("../../resources/test_key.pem");
    static PUB_KEY_BYTES: &[u8] = include_bytes!("../../resources/test_key.pub.pem");

    fn sync<F: Future>(future: F) -> <F as std::future::Future>::Output {
        tokio::runtime::Runtime::new().unwrap().block_on(future)
    }

    fn prepare_keys(jail: &mut Jail) {
        jail.create_file("key.pem", str::from_utf8(PRIV_KEY_BYTES).unwrap())
            .unwrap();
        jail.create_file("key.pub.pem", str::from_utf8(PUB_KEY_BYTES).unwrap())
            .unwrap();
        jail.set_env("POLAR_SECURITY.PRIVATE_KEY_PATH", "key.pem");
        jail.set_env("POLAR_SECURITY.PUBLIC_KEY_PATH", "key.pub.pem");
    }

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
        private_key_path: Option<String>,
        public_key_path: Option<String>,
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
                private_key_path,
                public_key_path,
                jwt_lifetime,
            }),
        }
    }

    fn empty_cli() -> Cli {
        cli(
            None, None, None, None, None, None, None, None, None, None, None, None,
        )
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
            Some("key.pem".to_string()),
            Some("key.pub.pem".to_string()),
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
        // assert_eq!(config.security.private_key, PRIV_KEY_BYTES);
        // assert_eq!(config.security.public_key, PUB_KEY_BYTES);
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
            None, // FIXME
            None, // FIXME
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
        // assert_eq!(config.security.jwt_secret, "secret");
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
            jail.create_file("key.pem", str::from_utf8(PRIV_KEY_BYTES).unwrap())
                .unwrap();
            jail.create_file("key.pub.pem", str::from_utf8(PUB_KEY_BYTES).unwrap())
                .unwrap();

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
                Some("key.pem".to_string()),
                Some("key.pub.pem".to_string()),
                None,
            );

            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "42.42.42.42"
            "#,
            )?;

            let config_result = sync(Config::figment(&args));
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
            None,
        );

        assert!(sync(Config::figment(&args)).is_err())
    }

    // Env

    #[test]
    fn env_select_profile() {
        Jail::expect_with(|jail| {
            prepare_keys(jail);

            jail.set_env("POLAR_PROFILE", "custom");
            let config = sync(Config::figment(&empty_cli())).unwrap();
            assert_eq!(config.profile(), "custom");

            Ok(())
        })
    }

    // Precedence

    #[test]
    fn precedence_env_over_file() {
        Jail::expect_with(|jail| {
            prepare_keys(jail);

            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "42.42.42.42"
                "#,
            )?;

            jail.set_env("POLAR_ADDRESS", "0.0.0.0");
            let figment = sync(Config::figment(&empty_cli())).unwrap();
            let config: Config = figment.extract()?;

            assert_eq!(config.address, "0.0.0.0");

            Ok(())
        })
    }

    #[test]
    fn precedence_args_over_file() {
        Jail::expect_with(|jail| {
            prepare_keys(jail);

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
                None,
            );

            jail.create_file(
                "polar.toml",
                r#"
                [default]
                address = "42.42.42.42"
                "#,
            )?;

            let config_result = sync(Config::figment(&args));
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
            prepare_keys(jail);

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
                None,
            );

            jail.set_env("POLAR_ADDRESS", "0.0.0.0");

            let figment = sync(Config::figment(&args)).unwrap();
            let config: Config = figment.extract()?;

            assert_eq!(config.address, "192.168.1.42");

            Ok(())
        })
    }

    #[test]
    fn precedence_args_over_env_over_file() {
        Jail::expect_with(|jail| {
            prepare_keys(jail);

            jail.set_env("POLAR_PROFILE", "dev");
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
                None,
            );
            let figment = sync(Config::figment(&args)).unwrap();
            let config: Config = figment.select("default").extract()?;

            assert_eq!(config.address, "3.3.3.3"); // Arg over env over file
            assert_eq!(config.port, 2222); // env over file
            assert_eq!(config.database.host, "1.1.1.1"); // file

            Ok(())
        })
    }

    #[test]
    fn correct_db_url() {
        Jail::expect_with(|jail| {
            prepare_keys(jail);

            jail.set_env("POLAR_PROFILE", "dev");
            jail.create_file(
                "polar.toml",
                r#"
                [dev.database]
                host = "database"
                user = "polar"
                password = "polar"
                schema = "polar"
                "#,
            )?;
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
                None,
            );
            let figment = sync(Config::figment(&args)).unwrap();
            let config: Config = figment.extract()?;

            assert_eq!(figment.profile(), "dev");
            assert_eq!(
                config.database.url(),
                "postgres://polar:polar@database:5432/polar"
            );
            Ok(())
        })
    }
}
