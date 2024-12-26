CREATE TYPE execution_outcome_status AS ENUM(
  'UNKNOWN',
  'FAILURE',
  'SUCCESS_VALUE',
  'SUCCESS_RECEIPT_ID'
);

CREATE TABLE "transactions" (
    "id" bigserial NOT NULL,
    "transaction_hash" text NOT NULL,
    "included_in_block_hash" text NOT NULL,
    "included_in_chunk_hash" text NOT NULL,
    "index_in_chunk" integer NOT NULL,
    "signer_account_id" text NOT NULL,
    "receiver_account_id" text NOT NULL,
    "status" execution_outcome_status NOT NULL,
    "converted_into_receipt_id" text NOT NULL,
    "receipt_conversion_gas_burnt" numeric(20,0),
    "receipt_conversion_tokens_burnt" numeric(45,0),
    "block_timestamp" timestamptz NOT NULL
);

SELECT create_hypertable('transactions', 'block_timestamp', chunk_time_interval => INTERVAL '1 hour');

SELECT enable_chunk_skipping('transactions', 'id');

CREATE INDEX transactions_id_idx ON transactions (id DESC);
CREATE INDEX transactions_signer_account_id_id_idx ON transactions (signer_account_id, id DESC);
CREATE INDEX transactions_receiver_account_id_id_idx ON transactions (receiver_account_id, id DESC);
CREATE INDEX transactions_block_timestamp_index_in_chunk_idx ON transactions (block_timestamp DESC, index_in_chunk DESC);
CREATE INDEX transactions_converted_into_receipt_id_idx ON transactions (converted_into_receipt_id DESC);
CREATE INDEX transactions_included_in_block_hash_idx ON transactions (included_in_block_hash DESC, id DESC);
CREATE INDEX transactions_included_in_chunk_hash_idx ON transactions (included_in_chunk_hash DESC, id DESC);

ALTER TABLE transactions SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'signer_account_id,receiver_account_id',
    timescaledb.compress_orderby = 'id DESC'
);

SELECT add_compression_policy('transactions', INTERVAL '1 hour');
