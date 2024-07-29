BEGIN;

CREATE TABLE new_memecooking_meme_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    meme_id BIGINT NOT NULL,
    owner TEXT NOT NULL,
    end_timestamp_ms BIGINT NOT NULL,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    decimals INT NOT NULL,
    total_supply DECIMAL NOT NULL,
    reference TEXT NOT NULL,
    reference_hash TEXT NOT NULL,
    deposit_token_id TEXT NOT NULL
);

SELECT create_hypertable('new_memecooking_meme_testnet', 'timestamp');

CREATE INDEX new_memecooking_meme_testnet_idx_meme_id ON new_memecooking_meme_testnet(meme_id);
CREATE INDEX new_memecooking_meme_testnet_idx_owner ON new_memecooking_meme_testnet(owner);

COMMIT;
