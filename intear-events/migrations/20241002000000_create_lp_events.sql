CREATE TABLE liquidity_pool_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    provider_account_id TEXT NOT NULL,
    tokens JSONB NOT NULL,
    pool_id TEXT NOT NULL
);

SELECT create_hypertable('liquidity_pool_testnet', 'timestamp');

CREATE INDEX liquidity_pool_testnet_idx_provider_account_id ON liquidity_pool_testnet(provider_account_id);
CREATE INDEX liquidity_pool_testnet_idx_pool_id ON liquidity_pool_testnet(pool_id);
CREATE INDEX liquidity_pool_testnet_idx_tokens ON liquidity_pool_testnet USING GIN(tokens jsonb_ops);
CREATE INDEX liquidity_pool_testnet_idx_id ON liquidity_pool_testnet(id);
CREATE INDEX liquidity_pool_testnet_idx_id_provider_account_id ON liquidity_pool_testnet(id, provider_account_id);
CREATE INDEX liquidity_pool_testnet_idx_id_pool_id ON liquidity_pool_testnet(id, pool_id);
CREATE INDEX liquidity_pool_testnet_idx_timestamp_provider_account_id ON liquidity_pool_testnet(timestamp, provider_account_id);
CREATE INDEX liquidity_pool_testnet_idx_timestamp_pool_id ON liquidity_pool_testnet(timestamp, pool_id);


CREATE TABLE liquidity_pool (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    provider_account_id TEXT NOT NULL,
    tokens JSONB NOT NULL,
    pool_id TEXT NOT NULL
);

SELECT create_hypertable('liquidity_pool', 'timestamp');

CREATE INDEX liquidity_pool_idx_provider_account_id ON liquidity_pool(provider_account_id);
CREATE INDEX liquidity_pool_idx_pool_id ON liquidity_pool(pool_id);
CREATE INDEX liquidity_pool_idx_tokens ON liquidity_pool USING GIN(tokens jsonb_ops);
CREATE INDEX liquidity_pool_idx_id ON liquidity_pool(id);
CREATE INDEX liquidity_pool_idx_id_provider_account_id ON liquidity_pool(id, provider_account_id);
CREATE INDEX liquidity_pool_idx_id_pool_id ON liquidity_pool(id, pool_id);
CREATE INDEX liquidity_pool_idx_timestamp_provider_account_id ON liquidity_pool(timestamp, provider_account_id);
CREATE INDEX liquidity_pool_idx_timestamp_pool_id ON liquidity_pool(timestamp, pool_id);
