use crate::result::{DatabaseError, Error};
use diesel_derive_enum::DbEnum;


#[derive(Debug, DbEnum)]
#[PgType = "REVOCATION"]
pub enum Revocation {
    Manual,
    Logout,
    Expired
}

impl TryFrom<&str> for Revocation {
    type Error = Error<'static>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "manual" => Ok(Revocation::Manual),
            "logout" => Ok(Revocation::Logout),
            "expired" => Ok(Revocation::Expired),
            _ => Err(DatabaseError::ParsingError.into())
        }
    }
}
