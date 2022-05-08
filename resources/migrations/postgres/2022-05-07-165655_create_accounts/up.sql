CREATE TABLE accounts (
    id                  SERIAL              PRIMARY KEY,

    firstname           VARCHAR(127)        NOT NULL,
    lastname            VARCHAR(127)        NOT NULL,
    email               VARCHAR(255)        NOT NULL        UNIQUE,
    bio                 TEXT                NOT NULL        DEFAULT '',
    picture_hash        VARCHAR(127)        NULL,

    created_at          TIMESTAMPTZ         NOT NULL        DEFAULT NOW(),
    updated_at          TIMESTAMPTZ         NOT NULL        DEFAULT NOW()
);

CREATE TABLE accounts_refresh_tokens (
    account_id          INTEGER             REFERENCES accounts (id),
    refresh_token_id    INTEGER             REFERENCES refresh_tokens (id),

    PRIMARY KEY (account_id, refresh_token_id)
);
