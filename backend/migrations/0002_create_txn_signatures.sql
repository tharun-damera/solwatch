-- Add migration script here
CREATE TABLE transaction_signatures (
    signature VARCHAR(88) PRIMARY KEY,
    account_address VARCHAR(44) NOT NULL,
    slot BIGINT NOT NULL,
    block_time BIGINT NOT NULL,
    confirmation_status VARCHAR(9) NOT NULL,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (account_address) REFERENCES accounts(address)
);
