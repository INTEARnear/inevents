BEGIN;

CREATE TABLE price_pool (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    pool_id TEXT NOT NULL,
    token0 TEXT NOT NULL,
    token1 TEXT NOT NULL,
    token0_in_1_token1 NUMERIC NOT NULL,
    token1_in_1_token0 NUMERIC NOT NULL,
    amount_in NUMERIC NOT NULL,
    amount_out NUMERIC NOT NULL
);

SELECT create_hypertable('price_pool', 'timestamp');

CREATE INDEX price_pool_idx_pool_id ON price_pool(pool_id);
CREATE INDEX price_pool_idx_token0 ON price_pool(token0);
CREATE INDEX price_pool_idx_token1 ON price_pool(token1);

CREATE TABLE price_token (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    token TEXT NOT NULL,
    price_usd NUMERIC NOT NULL,
    price_near NUMERIC NOT NULL
);

SELECT create_hypertable('price_token', 'timestamp');

CREATE INDEX price_token_idx_token ON price_token(token);

COMMIT;