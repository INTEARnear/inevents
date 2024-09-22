BEGIN;

CREATE TABLE memecooking_deposit (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    meme_id BIGINT NOT NULL,
    amount DECIMAL NOT NULL,
    protocol_fee DECIMAL NOT NULL,
    referrer TEXT,
    referrer_fee DECIMAL
);

SELECT create_hypertable('memecooking_deposit', 'timestamp');

CREATE INDEX memecooking_deposit_idx_meme_id ON memecooking_deposit(meme_id);
CREATE INDEX memecooking_deposit_idx_trader ON memecooking_deposit(trader);


CREATE TABLE memecooking_deposit_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    meme_id BIGINT NOT NULL,
    amount DECIMAL NOT NULL,
    protocol_fee DECIMAL NOT NULL,
    referrer TEXT,
    referrer_fee DECIMAL
);

SELECT create_hypertable('memecooking_deposit_testnet', 'timestamp');

CREATE INDEX memecooking_deposit_testnet_idx_meme_id ON memecooking_deposit_testnet(meme_id);
CREATE INDEX memecooking_deposit_testnet_idx_trader ON memecooking_deposit_testnet(trader);


CREATE TABLE memecooking_withdraw (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    meme_id BIGINT NOT NULL,
    amount DECIMAL NOT NULL,
    fee DECIMAL NOT NULL
);

SELECT create_hypertable('memecooking_withdraw', 'timestamp');

CREATE INDEX memecooking_withdraw_idx_meme_id ON memecooking_withdraw(meme_id);
CREATE INDEX memecooking_withdraw_idx_trader ON memecooking_withdraw(trader);


CREATE TABLE memecooking_withdraw_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    meme_id BIGINT NOT NULL,
    amount DECIMAL NOT NULL,
    fee DECIMAL NOT NULL
);

SELECT create_hypertable('memecooking_withdraw_testnet', 'timestamp');

CREATE INDEX memecooking_withdraw_testnet_idx_meme_id ON memecooking_withdraw_testnet(meme_id);
CREATE INDEX memecooking_withdraw_testnet_idx_trader ON memecooking_withdraw_testnet(trader);


CREATE TABLE memecooking_create_token (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    meme_id BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    total_supply DECIMAL NOT NULL,
    pool_id BIGINT NOT NULL
);

SELECT create_hypertable('memecooking_create_token', 'timestamp');

CREATE INDEX memecooking_create_token_idx_meme_id ON memecooking_create_token(meme_id);
CREATE INDEX memecooking_create_token_idx_token_id ON memecooking_create_token(token_id);


CREATE TABLE memecooking_create_token_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    meme_id BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    total_supply DECIMAL NOT NULL,
    pool_id BIGINT NOT NULL
);

SELECT create_hypertable('memecooking_create_token_testnet', 'timestamp');

CREATE INDEX memecooking_create_token_testnet_idx_meme_id ON memecooking_create_token_testnet(meme_id);
CREATE INDEX memecooking_create_token_testnet_idx_token_id ON memecooking_create_token_testnet(token_id);


COMMIT;
