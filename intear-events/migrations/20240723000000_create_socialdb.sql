BEGIN;

CREATE TABLE socialdb_index (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,

    account_id TEXT NOT NULL,
    index_type TEXT NOT NULL,
    index_key JSONB NOT NULL,
    index_value JSONB NOT NULL
);

SELECT create_hypertable('socialdb_index', 'timestamp');

CREATE INDEX socialdb_index_idx_account_id ON socialdb_index(account_id);
CREATE INDEX socialdb_index_idx_index_type ON socialdb_index(index_type);
CREATE INDEX socialdb_index_idx_index_key ON socialdb_index(index_key);

COMMIT;
