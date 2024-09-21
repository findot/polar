CREATE TABLE accounts(
    id                  SERIAL              PRIMARY KEY,

    alias               VARCHAR(127)        NOT NULL        UNIQUE,
    email               VARCHAR(255)        NOT NULL        UNIQUE,
    password_hash       VARCHAR(255)        NOT NULL,

    created_at          TIMESTAMPTZ         NOT NULL        DEFAULT NOW(),
    updated_at          TIMESTAMPTZ         NOT NULL        DEFAULT NOW(),

    blocked             BOOLEAN             NOT NULL        DEFAULT FALSE,
    block_date          TIMESTAMPTZ         NULL,
    block_reason        VARCHAR(255)        NULL,

    CHECK (updated_at >= created_at),
    CHECK(
        NOT blocked
        OR
        blocked AND block_date IS NOT NULL AND block_date >= created_at
    )
);

CREATE INDEX index_accounts_alias ON accounts (alias);

CREATE TABLE accounts_refresh_tokens(
    account_id          INTEGER             REFERENCES accounts (id),
    refresh_token_id    INTEGER             REFERENCES refresh_tokens (id),

    PRIMARY KEY (account_id, refresh_token_id)
);
