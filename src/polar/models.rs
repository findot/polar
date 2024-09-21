use chrono::{DateTime as ChronoDateTime, Utc};
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, DieselNewType)]
pub struct DateTime(ChronoDateTime<Utc>);

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.timestamp())
    }
}

#[derive(PartialEq, Debug, diesel_derive_enum::DbEnum, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::Revocation"]
pub enum Revocation {
    Manual,
    Logout,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::accounts)]
pub struct Account {
    pub id: i32,

    pub alias: String,
    pub email: String,
    pub password_hash: String,

    pub created_at: DateTime,
    pub updated_at: DateTime,

    pub blocked: bool,
    pub block_date: Option<DateTime>,
    pub block_reason: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::accounts)]
pub struct NewAccount {
    pub alias: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Serialize)]
#[diesel(table_name = crate::schema::accounts)]
pub struct AccountSummary {
    pub id: i32,

    pub alias: String,
    pub email: String,

    pub created_at: DateTime,
    pub updated_at: DateTime,

    pub blocked: bool,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshToken {
    pub id: i32,

    pub hash: String,
    pub issuance_date: DateTime,
    pub valid_until: DateTime,

    pub revoked: bool,
    pub revocation: Option<Revocation>,
    pub revocation_date: Option<DateTime>,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::accounts_refresh_tokens)]
#[diesel(belongs_to(Account))]
#[diesel(belongs_to(RefreshToken))]
#[diesel(primary_key(account_id, refresh_token_id))]
pub struct AccountRefreshToken {
    pub account_id: i32,
    pub refresh_token_id: i32,
}
