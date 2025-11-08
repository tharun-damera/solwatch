-- Add migration script here
CREATE TABLE accounts (
    address VARCHAR(44) PRIMARY KEY,
    lamports BIGINT NOT NULL,
    owner VARCHAR(44) NOT NULL,
    executable BOOLEAN NOT NULL,
    data_length BIGINT NOT NULL,
    rent_epoch BIGINT NOT NULL,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);