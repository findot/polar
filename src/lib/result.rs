use crate::lib::result::ConfigurationError::{MisconfiguredEntry, MissingEntry};
use diesel::ConnectionError;
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Error as IOError;
use std::option::Option;
use std::result::Result as StdResult;

use rocket::figment::Error as FigmentError;
use rocket::Error as RocketError;
use serde_json::Error as SerdeJsonError;
use serde_xml_rs::Error as SerdeXmlError;
use serde_yaml::Error as SerdeYamlError;
use toml::ser::Error as SerdeTomlError;

pub type Result<'a, T> = StdResult<T, Error<'a>>;

// ---------------------------------------------------------------------------- Configuration Error

#[derive(Debug)]
pub enum ConfigurationError<'a> {
    MissingEntry(&'a str),
    MisconfiguredEntry(&'a str),
    InvalidSource(IOError),
}

impl<'a> ConfigurationError<'a> {
    pub fn missing(key: &str) -> ConfigurationError {
        MissingEntry(key)
    }
    pub fn misconfigured(key: &str) -> ConfigurationError {
        MisconfiguredEntry(key)
    }
}

impl<'a> Display for ConfigurationError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConfigurationError::MisconfiguredEntry(key) => {
                write!(f, "Incorrect value for configuration key {}", key)
            }
            ConfigurationError::MissingEntry(key) => {
                write!(f, "Missing value for configuration key {}", key)
            }
            ConfigurationError::InvalidSource(key) => {
                write!(f, "Invalid source {}", key) // TODO - Check this error msg
            }
        }
    }
}

impl<'a> StdError for ConfigurationError<'a> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

// ---------------------------------------------------------------------------- Serialization Error

#[derive(Debug)]
pub enum SerdeError {
    JsonError(SerdeJsonError),
    YamlError(SerdeYamlError),
    TomlError(SerdeTomlError),
    XmlError(SerdeXmlError),
}

impl Display for SerdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SerdeError::JsonError(sje) => Display::fmt(&sje, f),
            SerdeError::YamlError(sye) => Display::fmt(&sye, f),
            SerdeError::TomlError(ste) => Display::fmt(&ste, f),
            SerdeError::XmlError(sxe) => Display::fmt(&sxe, f),
        }
    }
}

impl StdError for SerdeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SerdeError::JsonError(sje) => sje.source(),
            SerdeError::YamlError(sye) => sye.source(),
            SerdeError::TomlError(ste) => ste.source(),
            SerdeError::XmlError(sxe) => sxe.source(),
        }
    }
}

impl From<SerdeJsonError> for SerdeError {
    fn from(sje: SerdeJsonError) -> Self {
        Self::JsonError(sje)
    }
}

impl From<SerdeYamlError> for SerdeError {
    fn from(sye: SerdeYamlError) -> Self {
        Self::YamlError(sye)
    }
}

impl From<SerdeXmlError> for SerdeError {
    fn from(sxe: SerdeXmlError) -> Self {
        Self::XmlError(sxe)
    }
}

impl From<SerdeTomlError> for SerdeError {
    fn from(ste: SerdeTomlError) -> Self {
        Self::TomlError(ste)
    }
}

// --------------------------------------------------------------------------------- Database Error

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(ConnectionError),
    MigrationError(Box<dyn StdError + Send + Sync>),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DatabaseError::ConnectionError(ce) => Display::fmt(ce, f),
            DatabaseError::MigrationError(rme) => Display::fmt(rme, f),
        }
    }
}

impl StdError for DatabaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            DatabaseError::ConnectionError(ce) => ce.source(),
            DatabaseError::MigrationError(rme) => rme.source(),
        }
    }
}

impl From<ConnectionError> for DatabaseError {
    fn from(ce: ConnectionError) -> Self {
        DatabaseError::ConnectionError(ce)
    }
}

impl From<Box<dyn StdError + Send + Sync>> for DatabaseError {
    fn from(rme: Box<dyn StdError + Send + Sync>) -> Self {
        DatabaseError::MigrationError(rme)
    }
}

// -------------------------------------------------------------------------------- Root Error type

#[derive(Debug)]
pub enum Error<'a> {
    // Add error types here
    NotFound,
    ConfigurationError(ConfigurationError<'a>),
    FigmentError(FigmentError),
    SerdeError(SerdeError),
    RocketError(RocketError),
    DatabaseError(DatabaseError),
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            // Add error types here
            Error::NotFound => write!(f, "Not found"),
            Error::ConfigurationError(ce) => Display::fmt(&ce, f),
            Error::FigmentError(fe) => Display::fmt(&fe, f),
            Error::SerdeError(se) => Display::fmt(se, f),
            Error::RocketError(re) => Display::fmt(re, f),
            Error::DatabaseError(de) => Display::fmt(de, f),
        }
    }
}

impl<'a> StdError for Error<'a> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            // Add error types here
            Error::ConfigurationError(e) => e.source(),
            Error::FigmentError(e) => e.source(),
            Error::SerdeError(e) => e.source(),
            Error::RocketError(e) => e.source(),
            Error::DatabaseError(e) => e.source(),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------ Conversions

impl<'a> From<FigmentError> for Error<'a> {
    fn from(fe: FigmentError) -> Self {
        Error::FigmentError(fe)
    }
}

impl<'a, 'b: 'a> From<ConfigurationError<'b>> for Error<'a> {
    fn from(ce: ConfigurationError<'b>) -> Self {
        Error::ConfigurationError(ce)
    }
}

impl<'a> From<IOError> for ConfigurationError<'a> {
    fn from(e: IOError) -> Self {
        ConfigurationError::InvalidSource(e)
    }
}

// FIXME - Might be over-generalized (IOError might happen outside of configuration)
impl<'a> From<IOError> for Error<'a> {
    fn from(e: IOError) -> Self {
        Error::ConfigurationError(e.into())
    }
}

impl<'a> From<SerdeError> for Error<'a> {
    fn from(se: SerdeError) -> Self {
        Error::SerdeError(se)
    }
}

impl<'a> From<SerdeJsonError> for Error<'a> {
    fn from(sje: SerdeJsonError) -> Self {
        Error::SerdeError(SerdeError::JsonError(sje))
    }
}

impl<'a> From<SerdeYamlError> for Error<'a> {
    fn from(sye: SerdeYamlError) -> Self {
        Error::SerdeError(SerdeError::YamlError(sye))
    }
}

impl<'a> From<SerdeTomlError> for Error<'a> {
    fn from(ste: SerdeTomlError) -> Self {
        Error::SerdeError(SerdeError::TomlError(ste))
    }
}

impl<'a> From<SerdeXmlError> for Error<'a> {
    fn from(sxe: SerdeXmlError) -> Self {
        Error::SerdeError(SerdeError::XmlError(sxe))
    }
}

impl<'a> From<RocketError> for Error<'a> {
    fn from(re: RocketError) -> Self {
        Error::RocketError(re)
    }
}

impl<'a> From<DatabaseError> for Error<'a> {
    fn from(ce: DatabaseError) -> Self {
        Error::DatabaseError(ce)
    }
}
