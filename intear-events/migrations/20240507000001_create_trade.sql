BEGIN;

CREATE TABLE trade_pool (
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

SELECT create_hypertable('trade_pool', 'timestamp');

CREATE INDEX trade_pool_idx_trader ON trade_pool(trader);
CREATE INDEX trade_pool_idx_pool ON trade_pool(pool);
CREATE INDEX trade_pool_idx_token_in ON trade_pool(token_in);
CREATE INDEX trade_pool_idx_token_out ON trade_pool(token_out);

CREATE TABLE trade_swap (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    balance_changes JSONB NOT NULL
);

SELECT create_hypertable('trade_swap', 'timestamp');

CREATE INDEX trade_swap_idx_trader ON trade_swap(trader);
CREATE INDEX trade_swap_idx_balance_change_token ON trade_swap USING GIN(balance_changes jsonb_path_ops);

CREATE TABLE trade_pool_change (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    pool_id TEXT NOT NULL,
    pool JSONB NOT NULL
);

SELECT create_hypertable('trade_pool_change', 'timestamp');

CREATE INDEX trade_pool_change_idx_pool_id ON trade_pool_change(pool_id);

COMMIT;