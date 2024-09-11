BEGIN;

CREATE TABLE moretps_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    claimed_account_id TEXT NOT NULL,
    claimed_parent_account_id TEXT NOT NULL,
    round_account_id TEXT NOT NULL,
    round_parent_account_id TEXT NOT NULL,
    is_success BOOLEAN NOT NULL
);

SELECT create_hypertable('moretps_testnet', 'timestamp');

CREATE INDEX new_memecooking_meme_testnet_idx_claimed_account_id ON moretps_testnet(claimed_account_id);
CREATE INDEX new_memecooking_meme_testnet_idx_claimed_parent_account_id ON moretps_testnet(claimed_parent_account_id);
CREATE INDEX new_memecooking_meme_testnet_idx_round_account_id ON moretps_testnet(round_account_id);
CREATE INDEX new_memecooking_meme_testnet_idx_round_parent_account_id ON moretps_testnet(round_parent_account_id);

CREATE TABLE moretps (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    claimed_account_id TEXT NOT NULL,
    claimed_parent_account_id TEXT NOT NULL,
    round_account_id TEXT NOT NULL,
    round_parent_account_id TEXT NOT NULL,
    is_success BOOLEAN NOT NULL
);

SELECT create_hypertable('moretps', 'timestamp');

CREATE INDEX new_memecooking_meme_idx_claimed_account_id ON moretps(claimed_account_id);
CREATE INDEX new_memecooking_meme_idx_claimed_parent_account_id ON moretps(claimed_parent_account_id);
CREATE INDEX new_memecooking_meme_idx_round_account_id ON moretps(round_account_id);
CREATE INDEX new_memecooking_meme_idx_round_parent_account_id ON moretps(round_parent_account_id);


CREATE TABLE block_info_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    block_producer TEXT NOT NULL,
    transaction_count INT NOT NULL,
    receipt_count INT NOT NULL
);

SELECT create_hypertable('block_info_testnet', 'timestamp');

CREATE INDEX block_info_testnet_idx_block_height ON block_info_testnet(block_height);
CREATE INDEX block_info_testnet_idx_block_hash ON block_info_testnet(block_hash);
CREATE INDEX block_info_testnet_idx_block_producer ON block_info_testnet(block_producer);

CREATE TABLE block_info (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    block_producer TEXT NOT NULL,
    transaction_count INT NOT NULL,
    receipt_count INT NOT NULL
);

SELECT create_hypertable('block_info', 'timestamp');

CREATE INDEX block_info_idx_block_height ON block_info(block_height);
CREATE INDEX block_info_idx_block_hash ON block_info(block_hash);
CREATE INDEX block_info_idx_block_producer ON block_info(block_producer);

COMMIT;
