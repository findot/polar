use std::fmt::Debug;
use chrono::{DateTime, Utc};

use crate::lib::database::schema::{accounts, accounts_refresh_tokens};
use super::RefreshToken;


// ------------------------------------------ Accounts ------------------------------------------ //

#[derive(Debug, Identifiable, Queryable, AsChangeset)]
#[table_name = "accounts"]
pub struct Account {
    pub id: u32,

    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub bio: String,
    pub picture_hash: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

// -------------------------------------- Accounts Tokens --------------------------------------- //

#[derive(Debug, Identifiable, Queryable, Associations)]
#[primary_key(account_id, refresh_token_id)]
#[belongs_to(Account, foreign_key = "account_id")]
#[belongs_to(RefreshToken, foreign_key = "refresh_token_id")]
#[table_name = "accounts_refresh_tokens"]
pub struct AccountToken {
    pub account_id: u32,
    pub refresh_token_id: u32
}
