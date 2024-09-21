use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::option::Option;
use std::result::Result as StdResult;

use tokio_postgres::Error as TokioPgError;

pub type Result<'a, T> = StdResult<T, Error<'a>>;

// -------------------------------------------------------------------------- Error types re-export

pub mod errors {
    pub use argon2::password_hash::Error as PwdHashError;
    pub use diesel::result::Error as DieselError;
    pub use diesel::ConnectionError as DbConnectionError;
    pub use figment::Error as FigmentError;
    pub use pem::PemError;
    pub use rocket::tokio::io::Error as IOError;
    pub use rocket::Error as RocketError;
    pub use serde_json::Error as SerdeJsonError;
    pub use serde_xml_rs::Error as SerdeXmlError;
    pub use serde_yaml::Error as SerdeYamlError;
    pub use std::error::Error as StdError;
    pub use tokio_postgres::Error as TokioPgError;
    pub use toml::ser::Error as SerdeTomlError;
}

use errors::*;

// ---------------------------------------------------------------------------- Configuration Error

#[derive(Debug)]
pub enum ConfigurationError<'a> {
    MissingEntry(&'a str),
    MisconfiguredEntry(&'a str),
    InvalidSource(IOError),
}

impl<'a> ConfigurationError<'a> {
    pub fn missing(key: &str) -> ConfigurationError {
        ConfigurationError::MissingEntry(key)
    }
    pub fn misconfigured(key: &str) -> ConfigurationError {
        ConfigurationError::MisconfiguredEntry(key)
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

// IOError -> ConfigurationError
impl<'a> From<IOError> for ConfigurationError<'a> {
    fn from(e: IOError) -> Self {
        ConfigurationError::InvalidSource(e)
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
    ConnectionError(DbConnectionError),
    MigrationError(Box<dyn StdError + Send + Sync>),
    TokioPgError(TokioPgError),
    DieselError(DieselError),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DatabaseError::ConnectionError(ce) => Display::fmt(ce, f),
            DatabaseError::MigrationError(rme) => Display::fmt(rme, f),
            DatabaseError::TokioPgError(tpge) => Display::fmt(tpge, f),
            DatabaseError::DieselError(de) => Display::fmt(de, f),
        }
    }
}

impl StdError for DatabaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            DatabaseError::ConnectionError(ce) => ce.source(),
            DatabaseError::MigrationError(rme) => rme.source(),
            DatabaseError::TokioPgError(tpge) => tpge.source(),
            DatabaseError::DieselError(de) => de.source(),
        }
    }
}

impl From<DbConnectionError> for DatabaseError {
    fn from(ce: DbConnectionError) -> Self {
        DatabaseError::ConnectionError(ce)
    }
}

impl From<Box<dyn StdError + Send + Sync>> for DatabaseError {
    fn from(rme: Box<dyn StdError + Send + Sync>) -> Self {
        DatabaseError::MigrationError(rme)
    }
}

impl From<TokioPgError> for DatabaseError {
    fn from(tpge: TokioPgError) -> Self {
        DatabaseError::TokioPgError(tpge)
    }
}

impl From<DieselError> for DatabaseError {
    fn from(de: DieselError) -> Self {
        DatabaseError::DieselError(de)
    }
}

// ----------------------------------------------------------------------------------- Crypto Error

#[derive(Debug)]
pub enum CryptoError {
    IOError(IOError),
    PemError(PemError),
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            CryptoError::IOError(ioe) => Display::fmt(ioe, f),
            CryptoError::PemError(pe) => Display::fmt(pe, f),
        }
    }
}

impl StdError for CryptoError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CryptoError::IOError(ioe) => ioe.source(),
            CryptoError::PemError(pe) => pe.source(),
        }
    }
}

impl From<IOError> for CryptoError {
    fn from(value: IOError) -> Self {
        Self::IOError(value)
    }
}

impl From<PemError> for CryptoError {
    fn from(value: PemError) -> Self {
        Self::PemError(value)
    }
}

// ------------------------------------------------------------------------------------- Auth Error

#[derive(Debug)]
pub enum AuthError<'a> {
    Forbidden(&'a str),
    Unauthorized(&'a str),
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
    PwdHashError(PwdHashError),
    CryptoError(CryptoError),
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
            Error::PwdHashError(phe) => Display::fmt(phe, f),
            Error::CryptoError(ce) => Display::fmt(ce, f),
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
            //Error::PwdHashError(phe) => phe.source(),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------ Conversions

// FigmenError -> Error
impl<'a> From<FigmentError> for Error<'a> {
    fn from(fe: FigmentError) -> Self {
        Error::FigmentError(fe)
    }
}

// ConfigurationError -> Error
impl<'a, 'b: 'a> From<ConfigurationError<'b>> for Error<'a> {
    fn from(ce: ConfigurationError<'b>) -> Self {
        Error::ConfigurationError(ce)
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
        match ce {
            DatabaseError::DieselError(diesel::NotFound) => Self::NotFound,
            _ => Error::DatabaseError(ce),
        }
    }
}

impl<'a> From<PwdHashError> for Error<'a> {
    fn from(phe: PwdHashError) -> Self {
        Error::PwdHashError(phe)
    }
}

impl<'a> From<DieselError> for Error<'a> {
    fn from(de: DieselError) -> Self {
        Error::DatabaseError(de.into())
    }
}

impl<'a> From<CryptoError> for Error<'a> {
    fn from(ce: CryptoError) -> Self {
        Error::CryptoError(ce)
    }
}
