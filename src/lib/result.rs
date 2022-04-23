use crate::lib::result::ConfigurationError::{MisconfiguredEntry, MissingEntry};
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Error as IOError;
use std::option::Option;
use std::result::Result as StdResult;

use rocket::figment::Error as FigmentError;

pub type Result<'a, T> = StdResult<T, Error<'a>>;

// ---------------------------------------------------------------------------- Configuration error

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

// -------------------------------------------------------------------------------- Root Error type

#[derive(Debug)]
pub enum Error<'a> {
    // Add error types here
    NotFound,
    ConfigurationError(ConfigurationError<'a>),
    FigmentError(FigmentError),
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            // Add error types here
            Error::NotFound => write!(f, "Not found"),
            Error::ConfigurationError(ce) => std::fmt::Display::fmt(&ce, f),
            Error::FigmentError(fe) => std::fmt::Display::fmt(&fe, f),
        }
    }
}

impl<'a> StdError for Error<'a> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            // Add error types here
            Error::ConfigurationError(e) => e.source(),
            Error::FigmentError(e) => e.source(),
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
