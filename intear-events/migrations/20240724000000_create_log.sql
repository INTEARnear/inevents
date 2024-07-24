BEGIN;

CREATE TABLE log_nep297 (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,

    account_id TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    event_standard TEXT NOT NULL,
    event_version TEXT NOT NULL,
    event_event TEXT NOT NULL,
    event_data JSONB
);

SELECT create_hypertable('log_nep297', 'timestamp');

CREATE INDEX log_nep297_idx_account_id ON log_nep297(account_id);
CREATE INDEX log_nep297_idx_predecessor_id ON log_nep297(predecessor_id);
CREATE INDEX log_nep297_idx_event_standard ON log_nep297(event_standard);
CREATE INDEX log_nep297_idx_event_version ON log_nep297(event_version);
CREATE INDEX log_nep297_idx_event_event ON log_nep297(event_event);

CREATE TABLE log_text (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,

    account_id TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    log_text TEXT NOT NULL
);

SELECT create_hypertable('log_text', 'timestamp');

CREATE INDEX log_text_idx_account_id ON log_text(account_id);
CREATE INDEX log_text_idx_predecessor_id ON log_text(predecessor_id);

COMMIT;
