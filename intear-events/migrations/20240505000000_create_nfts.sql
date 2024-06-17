BEGIN;

CREATE TABLE nft_transfer (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    contract_id TEXT NOT NULL,
    old_owner_id TEXT NOT NULL,
    new_owner_id TEXT NOT NULL,
    token_ids TEXT[] NOT NULL,
    token_prices_near NUMERIC[] NULL,
    memo TEXT
);

SELECT create_hypertable('nft_transfer', 'timestamp');

CREATE INDEX nft_transfer_idx_old_owner_id ON nft_transfer(old_owner_id);
CREATE INDEX nft_transfer_idx_new_owner_id ON nft_transfer(new_owner_id);
CREATE INDEX nft_transfer_idx_contract_id ON nft_transfer(contract_id);

CREATE TABLE nft_mint (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    contract_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    token_ids TEXT[] NOT NULL,
    token_prices_near NUMERIC[] NULL,
    memo TEXT
);

SELECT create_hypertable('nft_mint', 'timestamp');

CREATE INDEX nft_mint_idx_owner_id ON nft_mint(owner_id);
CREATE INDEX nft_mint_idx_contract_id ON nft_mint(contract_id);

CREATE TABLE nft_burn (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    contract_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    token_ids TEXT[] NOT NULL,
    token_prices_near NUMERIC[] NULL,
    memo TEXT
);

SELECT create_hypertable('nft_burn', 'timestamp');

CREATE INDEX nft_burn_idx_owner_id ON nft_burn(owner_id);
CREATE INDEX nft_burn_idx_contract_id ON nft_burn(contract_id);

COMMIT;
