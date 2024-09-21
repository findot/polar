// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "revocation"))]
    pub struct Revocation;
}

diesel::table! {
    accounts (id) {
        id -> Int4,
        #[max_length = 127]
        alias -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        blocked -> Bool,
        block_date -> Nullable<Timestamptz>,
        #[max_length = 255]
        block_reason -> Nullable<Varchar>,
    }
}

diesel::table! {
    accounts_refresh_tokens (account_id, refresh_token_id) {
        account_id -> Int4,
        refresh_token_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Revocation;

    refresh_tokens (id) {
        id -> Int4,
        #[max_length = 127]
        hash -> Varchar,
        issuance_date -> Timestamptz,
        valid_until -> Timestamptz,
        revoked -> Bool,
        revocation -> Nullable<Revocation>,
        revocation_date -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(accounts_refresh_tokens -> accounts (account_id));
diesel::joinable!(accounts_refresh_tokens -> refresh_tokens (refresh_token_id));

diesel::allow_tables_to_appear_in_same_query!(accounts, accounts_refresh_tokens, refresh_tokens,);
