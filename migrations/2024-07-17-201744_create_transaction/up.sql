-- Your SQL goes here
CREATE TABLE chain_transactions (
    id SERIAL PRIMARY KEY,
    signature VARCHAR NOT NULL,
    fee BIGINT NOT NULL,
    slot INTEGER NOT NULL
);
