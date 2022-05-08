
table! {
    accounts (id) {
        id -> Int4,
        firstname -> Varchar,
        lastname -> Varchar,
        email -> Varchar,
        bio -> Text,
        picture_hash -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    accounts_refresh_tokens (account_id, refresh_token_id) {
        account_id -> Int4,
        refresh_token_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::database::types::RevocationMapping;

    refresh_tokens (id) {
        id -> Int4,
        hash -> Varchar,
        issuance_date -> Timestamptz,
        revoked -> Bool,
        revocation -> Nullable<RevocationMapping>,
        revocation_date -> Nullable<Timestamptz>,
    }
}

joinable!(accounts_refresh_tokens -> accounts (account_id));
joinable!(accounts_refresh_tokens -> refresh_tokens (refresh_token_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    accounts_refresh_tokens,
    refresh_tokens,
);
