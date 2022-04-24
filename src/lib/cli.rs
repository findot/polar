use std::collections::BTreeMap;
use clap::{Args, Parser, Subcommand};
use figment::{
    Error as FigmentError,
    map,
    Metadata,
    Profile,
    Provider,
    value::{Dict, Map, Value},
};

/* -------------------------------------- Util functions --------------------------------------- */

#[inline]
fn ref_str(s: &Option<String>) -> Option<&str> {
    s.as_ref().map(String::as_str)
}

#[inline]
fn filter_none<K: Ord, V>(mut dict: BTreeMap<K, Option<V>>) -> BTreeMap<K, V> {
    dict.retain(|_, v| v.is_some());
    dict.into_iter().map(|(k, v)| (k, v.unwrap())).collect()
}

#[inline]
fn migrate_data<'a>(data: &Migrate) -> Map<&'a str, Value> {
    filter_none(map! {
        "host" => ref_str(&data.database_host).map(Value::from),
        "port" => data.database_port.map(Value::from),
        "user" => ref_str(&data.database_user).map(Value::from),
        "password" => ref_str(&data.database_password).map(Value::from),
        "schema" => ref_str(&data.database_schema).map(Value::from)
    })
}

#[inline]
fn serve_data<'a>(data: &Serve) -> Map<&'a str, Value> {
    let database = map! {
        "host" => ref_str(&data.database_host).map(Value::from),
        "port" => data.database_port.map(Value::from),
        "user" => ref_str(&data.database_user).map(Value::from),
        "password" => ref_str(&data.database_password).map(Value::from),
        "schema" => ref_str(&data.database_schema).map(Value::from)
    };

    let security = map! {
        "jwt_secret" => ref_str(&data.jwt_secret).map(Value::from),
        "jwt_lifetime" => data.jwt_lifetime.map(Value::from)
    };

     filter_none(map! {
        "address" => ref_str(&data.address).map(Value::from),
        "port" => data.port.map(Value::from),
        "database" => Some(filter_none(database).into()),
        "security" => Some(filter_none(security).into()),
    })

}


/* --------------------------------------- Args Parsing ---------------------------------------- */

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Configuration file path
    #[clap(short = 'C', long)]
    pub configuration: Option<String>,

    /// Configuration profile to use
    #[clap(short = 'P', long)]
    pub profile: Option<String>,

    #[clap(subcommand)]
    pub command: Command
}

#[derive(Args)]
pub struct Migrate {
    /// Database IP address to connect to
    #[clap(long, short = 'd')]
    pub database_host: Option<String>,

    /// Database port number to connect to
    #[clap(long, short = 'n')]
    pub database_port: Option<u16>,

    /// Username with which polar will authenticate to the database
    #[clap(long, short = 'u')]
    pub database_user: Option<String>,

    /// Password with which polar will authenticate to the database
    #[clap(long, short = 'w')]
    pub database_password: Option<String>,

    /// Database schema to use
    #[clap(long, short = 's')]
    pub database_schema: Option<String>,
}

#[derive(Args)]
pub struct Serve {
    /// IP address to bind to
    #[clap(short, long)]
    pub address: Option<String>,

    /// Port number to use for connection or 0 for default
    #[clap(short, long)]
    pub port: Option<u16>,

    /// Database IP address to connect to
    #[clap(short = 'd', long)]
    pub database_host: Option<String>,

    /// Database port number to connect to
    #[clap(short = 'n', long)]
    pub database_port: Option<u16>,

    /// Username with which polar will authenticate to the database
    #[clap(short = 'u', long)]
    pub database_user: Option<String>,

    /// Password with which polar will authenticate to the database
    #[clap(short = 'w', long)]
    pub database_password: Option<String>,

    /// Database schema to use
    #[clap(short = 's', long)]
    pub database_schema: Option<String>,

    /// Seed of the jwt generation
    #[clap(short = 'k', long)]
    pub jwt_secret: Option<String>,

    /// Lifespan (in seconds) during which any emitted jwt token will be valid
    #[clap(short = 'l', long)]
    pub jwt_lifetime: Option<u16>,
}

#[derive(Subcommand)]
pub enum Command {
    Migrate(Migrate),
    Serve(Serve)
}

impl Provider for Cli {
    fn metadata(&self) -> Metadata {
        Metadata::named("Arguments")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, FigmentError> {
        let data = match &self.command {
            Command::Migrate(migrate) => migrate_data(migrate),
            Command::Serve(serve) => serve_data(serve)
        }.into_iter()
         .map(|(k, v)| (k.to_string(), v))
         .collect();

        let profile_str = ref_str(&self.profile).unwrap_or("default");
        let profile = Profile::from(profile_str);

        Ok(map![profile => data])
    }

    fn profile(&self) -> Option<Profile> {
        self.profile.as_ref().map(|p| Profile::new(p.as_str()))
    }
}