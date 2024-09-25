-- no-transaction

CREATE MATERIALIZED VIEW price_token_1day_ohlc
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', timestamp) AS bucket,
    token,
    FIRST(price_usd, timestamp) AS open,
    MAX(price_usd) AS high,
    MIN(price_usd) AS low,
    LAST(price_usd, timestamp) AS close
FROM price_token
GROUP BY bucket, token;

SELECT add_continuous_aggregate_policy('price_token_1day_ohlc',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '10 seconds',
    schedule_interval => INTERVAL '10 seconds');
