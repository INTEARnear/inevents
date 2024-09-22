BEGIN;

CREATE TABLE trade_pool_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    pool TEXT NOT NULL,
    token_in TEXT NOT NULL,
    token_out TEXT NOT NULL,
    amount_in NUMERIC NOT NULL,
    amount_out NUMERIC NOT NULL
);

SELECT create_hypertable('trade_pool_testnet', 'timestamp');

CREATE INDEX trade_pool_testnet_idx_trader ON trade_pool_testnet(trader);
CREATE INDEX trade_pool_testnet_idx_pool ON trade_pool_testnet(pool);
CREATE INDEX trade_pool_testnet_idx_token_in ON trade_pool_testnet(token_in);
CREATE INDEX trade_pool_testnet_idx_token_out ON trade_pool_testnet(token_out);

CREATE TABLE trade_swap_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    balance_changes JSONB NOT NULL
);

SELECT create_hypertable('trade_swap_testnet', 'timestamp');

CREATE INDEX trade_swap_testnet_idx_trader ON trade_swap_testnet(trader);
CREATE INDEX trade_swap_testnet_idx_balance_change_token ON trade_swap_testnet USING GIN(balance_changes jsonb_path_ops);

CREATE TABLE trade_pool_change_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    pool_id TEXT NOT NULL,
    pool JSONB NOT NULL
);

SELECT create_hypertable('trade_pool_change_testnet', 'timestamp');

CREATE INDEX trade_pool_change_testnet_idx_pool_id ON trade_pool_change_testnet(pool_id);

CREATE TABLE newcontract_nep141_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    account_id TEXT NOT NULL
);

SELECT create_hypertable('newcontract_nep141_testnet', 'timestamp');

CREATE INDEX newcontract_nep141_testnet_idx_account_id ON newcontract_nep141_testnet(account_id);

COMMIT;
