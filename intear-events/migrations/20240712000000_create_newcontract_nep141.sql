BEGIN;

CREATE TABLE newcontract_nep141 (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    account_id TEXT NOT NULL
);

SELECT create_hypertable('newcontract_nep141', 'timestamp');

CREATE INDEX newcontract_nep141_idx_account_id ON newcontract_nep141(account_id);

COMMIT;
