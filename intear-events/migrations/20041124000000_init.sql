CREATE TABLE nft_transfer (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    contract_id TEXT NOT NULL,
    old_owner_id TEXT NOT NULL,
    new_owner_id TEXT NOT NULL,
    token_ids TEXT[] NOT NULL,
    token_prices_near NUMERIC[] NOT NULL,
    memo TEXT
);

SELECT create_hypertable('nft_transfer', 'timestamp');

CREATE TABLE nft_mint (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    contract_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    token_ids TEXT[] NOT NULL,
    memo TEXT
);

SELECT create_hypertable('nft_mint', 'timestamp');

CREATE TABLE nft_burn (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    contract_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    token_ids TEXT[] NOT NULL,
    memo TEXT
);

SELECT create_hypertable('nft_burn', 'timestamp');

CREATE TABLE potlock_donation (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    donation_id BIGINT NOT NULL,
    donor_id TEXT NOT NULL,
    total_amount NUMERIC NOT NULL,
    message TEXT,
    donated_at TIMESTAMPTZ NOT NULL,
    project_id TEXT NOT NULL,
    protocol_fee NUMERIC NOT NULL,
    referrer_id TEXT,
    referrer_fee NUMERIC
);

SELECT create_hypertable('potlock_donation', 'timestamp');

CREATE TABLE potlock_pot_project_donation (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    donation_id BIGINT NOT NULL,
    pot_id TEXT NOT NULL,
    donor_id TEXT NOT NULL,
    total_amount NUMERIC NOT NULL,
    net_amount NUMERIC NOT NULL,
    message TEXT,
    donated_at TIMESTAMPTZ NOT NULL,
    project_id TEXT NOT NULL,
    referrer_id TEXT,
    referrer_fee NUMERIC,
    protocol_fee NUMERIC NOT NULL,
    chef_id TEXT,
    chef_fee NUMERIC
);

SELECT create_hypertable('potlock_pot_project_donation', 'timestamp');

CREATE TABLE potlock_pot_donation (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    donation_id BIGINT NOT NULL,
    pot_id TEXT NOT NULL,
    donor_id TEXT NOT NULL,
    total_amount NUMERIC NOT NULL,
    net_amount NUMERIC NOT NULL,
    message TEXT,
    donated_at TIMESTAMPTZ NOT NULL,
    referrer_id TEXT,
    referrer_fee NUMERIC,
    protocol_fee NUMERIC NOT NULL,
    chef_id TEXT,
    chef_fee NUMERIC,
    ft_id TEXT NOT NULL DEFAULT 'near'
);

SELECT create_hypertable('potlock_pot_donation', 'timestamp');

CREATE TABLE trade_pool (
    id BIGSERIAL,
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

CREATE TABLE trade_swap (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    trader TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    balance_changes JSONB NOT NULL
);

SELECT create_hypertable('trade_swap', 'timestamp');

CREATE TABLE trade_pool_change (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    pool_id TEXT NOT NULL,
    pool JSONB NOT NULL
);

SELECT create_hypertable('trade_pool_change', 'timestamp');

CREATE TABLE price_token (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    token TEXT NOT NULL,
    price_usd NUMERIC NOT NULL
);

SELECT create_hypertable('price_token', 'timestamp');

CREATE TABLE newcontract_nep141 (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    account_id TEXT NOT NULL
);

SELECT create_hypertable('newcontract_nep141', 'timestamp');

CREATE TABLE newcontract_nep171 (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    account_id TEXT NOT NULL
);

SELECT create_hypertable('newcontract_nep171', 'timestamp');

CREATE TABLE socialdb_index (
    id BIGSERIAL,
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

CREATE TABLE log_nep297 (
    id BIGSERIAL,
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

CREATE TABLE log_text (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    account_id TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    log_text TEXT NOT NULL
);

SELECT create_hypertable('log_text', 'timestamp');

CREATE TABLE new_memecooking_meme (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    meme_id BIGINT NOT NULL,
    owner TEXT NOT NULL,
    end_timestamp_ms BIGINT NOT NULL,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    decimals INT NOT NULL,
    total_supply DECIMAL NOT NULL,
    reference TEXT NOT NULL,
    reference_hash TEXT NOT NULL,
    deposit_token_id TEXT NOT NULL,
    soft_cap NUMERIC NOT NULL,
    hard_cap NUMERIC
);

SELECT create_hypertable('new_memecooking_meme', 'timestamp');

CREATE TABLE block_info (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    block_producer TEXT NOT NULL,
    transaction_count INT NOT NULL,
    receipt_count INT NOT NULL
);

CREATE TABLE tx_transactions (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    signer_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    receiver_id TEXT NOT NULL,
    priority_fee BIGINT,
    signature TEXT NOT NULL,
    transaction_id TEXT NOT NULL
);

SELECT create_hypertable('tx_transactions', 'timestamp');

CREATE TABLE tx_receipts (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    transaction_id TEXT NOT NULL,
    executor_id TEXT NOT NULL,
    success BOOLEAN
);

SELECT create_hypertable('tx_receipts', 'timestamp');

CREATE TABLE ft_transfer (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    old_owner_id TEXT NOT NULL,
    new_owner_id TEXT NOT NULL,
    memo TEXT
);

SELECT create_hypertable('ft_transfer', 'timestamp');

CREATE TABLE ft_mint (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    memo TEXT
);

SELECT create_hypertable('ft_mint', 'timestamp');

CREATE TABLE ft_burn (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    memo TEXT
);

SELECT create_hypertable('ft_burn', 'timestamp');

CREATE TABLE liquidity_add_remove (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    provider_account_id TEXT NOT NULL,
    tokens JSONB NOT NULL,
    pool_id TEXT NOT NULL
);

SELECT create_hypertable('liquidity_add_remove', 'timestamp');
