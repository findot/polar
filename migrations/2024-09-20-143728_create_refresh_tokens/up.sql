CREATE TYPE REVOCATION AS ENUM ('manual', 'logout');

CREATE TABLE refresh_tokens (
    id                  SERIAL              PRIMARY KEY,

    hash                VARCHAR (127)       NOT NULL,
    issuance_date       TIMESTAMPTZ         NOT NULL    DEFAULT NOW(),
    valid_until         TIMESTAMPTZ         NOT NULL,

    revoked             BOOLEAN             NOT NULL    DEFAULT FALSE,
    revocation          REVOCATION          NULL,
    revocation_date     TIMESTAMPTZ         NULL,

    CHECK (
        (revoked AND revocation IS NOT NULL AND revocation_date IS NOT NULL)
        OR
        (NOT revoked AND revocation IS NULL AND revocation_date IS NULL)
    ),
    CHECK (
        valid_until >= issuance_date
        AND
        (revocation_date IS NULL OR revocation_date >= issuance_date)
    )
);

CREATE INDEX index_refresh_tokens_hash ON refresh_tokens (hash);
