CREATE TABLE tx_transactions_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    signer_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    receiver_id TEXT NOT NULL,
    actions JSONB[] NOT NULL,
    priority_fee BIGINT,
    signature TEXT NOT NULL,
    transaction_id TEXT NOT NULL
);

SELECT create_hypertable('tx_transactions_testnet', 'timestamp');

CREATE INDEX tx_transactions_testnet_idx_signer_id ON tx_transactions_testnet (signer_id);
CREATE INDEX tx_transactions_testnet_idx_receiver_id ON tx_transactions_testnet (receiver_id);
CREATE INDEX tx_transactions_testnet_idx_transaction_id ON tx_transactions_testnet (transaction_id);
CREATE INDEX tx_transactions_testnet_idx_public_key ON tx_transactions_testnet (public_key);

CREATE TABLE tx_transactions (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    signer_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    receiver_id TEXT NOT NULL,
    actions JSONB[] NOT NULL,
    priority_fee BIGINT,
    signature TEXT NOT NULL,
    transaction_id TEXT NOT NULL
);

SELECT create_hypertable('tx_transactions', 'timestamp');

CREATE INDEX tx_transactions_idx_signer_id ON tx_transactions (signer_id);
CREATE INDEX tx_transactions_idx_receiver_id ON tx_transactions (receiver_id);
CREATE INDEX tx_transactions_idx_transaction_id ON tx_transactions (transaction_id);
CREATE INDEX tx_transactions_idx_public_key ON tx_transactions (public_key);


CREATE TABLE tx_receipts_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    transaction_id TEXT NOT NULL,
    executor_id TEXT NOT NULL,
    success BOOLEAN
);

SELECT create_hypertable('tx_receipts_testnet', 'timestamp');

CREATE INDEX tx_receipts_testnet_idx_receipt_id ON tx_receipts_testnet (receipt_id);
CREATE INDEX tx_receipts_testnet_idx_transaction_id ON tx_receipts_testnet (transaction_id);
CREATE INDEX tx_receipts_testnet_idx_executor_id ON tx_receipts_testnet (executor_id);

CREATE TABLE tx_receipts (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    transaction_id TEXT NOT NULL,
    executor_id TEXT NOT NULL,
    success BOOLEAN
);

SELECT create_hypertable('tx_receipts', 'timestamp');

CREATE INDEX tx_receipts_idx_receipt_id ON tx_receipts (receipt_id);
CREATE INDEX tx_receipts_idx_transaction_id ON tx_receipts (transaction_id);
CREATE INDEX tx_receipts_idx_executor_id ON tx_receipts (executor_id);
