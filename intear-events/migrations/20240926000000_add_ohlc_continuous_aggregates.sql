-- no-transaction

CREATE MATERIALIZED VIEW price_token_1min_ohlc
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', timestamp) AS bucket,
    token,
    FIRST(price_usd, timestamp) AS open,
    MAX(price_usd) AS high,
    MIN(price_usd) AS low,
    LAST(price_usd, timestamp) AS close
FROM price_token
GROUP BY bucket, token;

CREATE MATERIALIZED VIEW price_token_1hour_ohlc
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', timestamp) AS bucket,
    token,
    FIRST(price_usd, timestamp) AS open,
    MAX(price_usd) AS high,
    MIN(price_usd) AS low,
    LAST(price_usd, timestamp) AS close
FROM price_token
GROUP BY bucket, token;

SELECT add_continuous_aggregate_policy('price_token_1min_ohlc',
    start_offset => INTERVAL '5 minutes',
    end_offset => INTERVAL '1 second',
    schedule_interval => INTERVAL '1 second');

SELECT add_continuous_aggregate_policy('price_token_1hour_ohlc',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '5 seconds',
    schedule_interval => INTERVAL '5 seconds');
