use std::fmt::Debug;
use chrono::{DateTime, Utc};

use crate::database::schema::refresh_tokens;
use crate::database::types::Revocation;


// --------------------------------------- Refresh Token ---------------------------------------- //

#[derive(Debug, Identifiable, Queryable, AsChangeset)]
#[table_name = "refresh_tokens"]
pub struct RefreshToken {
    pub id: u32,

    pub hash: String,
    pub issuance_date: DateTime<Utc>,

    pub revoked: bool,
    pub revocation: Option<Revocation>,
    pub revocation_date: Option<DateTime<Utc>>
}