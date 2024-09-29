CREATE TABLE ft_transfer_testnet (
    id SERIAL,
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

SELECT create_hypertable('ft_transfer_testnet', 'timestamp');

CREATE INDEX ft_transfer_testnet_idx_old_owner_id ON ft_transfer_testnet(old_owner_id);
CREATE INDEX ft_transfer_testnet_idx_new_owner_id ON ft_transfer_testnet(new_owner_id);
CREATE INDEX ft_transfer_testnet_idx_token_id ON ft_transfer_testnet(token_id);
CREATE INDEX ft_transfer_testnet_idx_id ON ft_transfer_testnet(id);
CREATE INDEX ft_transfer_testnet_idx_id_old_owner_id ON ft_transfer_testnet(id, old_owner_id);
CREATE INDEX ft_transfer_testnet_idx_id_new_owner_id ON ft_transfer_testnet(id, new_owner_id);
CREATE INDEX ft_transfer_testnet_idx_id_token_id ON ft_transfer_testnet(id, token_id);
CREATE INDEX ft_transfer_testnet_idx_timestamp_old_owner_id ON ft_transfer_testnet(timestamp, old_owner_id);
CREATE INDEX ft_transfer_testnet_idx_timestamp_new_owner_id ON ft_transfer_testnet(timestamp, new_owner_id);
CREATE INDEX ft_transfer_testnet_idx_timestamp_token_id ON ft_transfer_testnet(timestamp, token_id);
CREATE INDEX ft_transfer_testnet_idx_token_id_amount ON ft_transfer_testnet(token_id, amount);
CREATE INDEX ft_transfer_testnet_idx_id_token_id_amount ON ft_transfer_testnet(id, token_id, amount);
CREATE INDEX ft_transfer_testnet_idx_timestamp_token_id_amount ON ft_transfer_testnet(timestamp, token_id, amount);

CREATE TABLE ft_transfer (
    id SERIAL,
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

CREATE INDEX ft_transfer_idx_old_owner_id ON ft_transfer(old_owner_id);
CREATE INDEX ft_transfer_idx_new_owner_id ON ft_transfer(new_owner_id);
CREATE INDEX ft_transfer_idx_token_id ON ft_transfer(token_id);
CREATE INDEX ft_transfer_idx_id ON ft_transfer(id);
CREATE INDEX ft_transfer_idx_id_old_owner_id ON ft_transfer(id, old_owner_id);
CREATE INDEX ft_transfer_idx_id_new_owner_id ON ft_transfer(id, new_owner_id);
CREATE INDEX ft_transfer_idx_id_token_id ON ft_transfer(id, token_id);
CREATE INDEX ft_transfer_idx_timestamp_old_owner_id ON ft_transfer(timestamp, old_owner_id);
CREATE INDEX ft_transfer_idx_timestamp_new_owner_id ON ft_transfer(timestamp, new_owner_id);
CREATE INDEX ft_transfer_idx_timestamp_token_id ON ft_transfer(timestamp, token_id);
CREATE INDEX ft_transfer_idx_token_id_amount ON ft_transfer(token_id, amount);
CREATE INDEX ft_transfer_idx_id_token_id_amount ON ft_transfer(id, token_id, amount);
CREATE INDEX ft_transfer_idx_timestamp_token_id_amount ON ft_transfer(timestamp, token_id, amount);


CREATE TABLE ft_mint_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    memo TEXT
);

SELECT create_hypertable('ft_mint_testnet', 'timestamp');

CREATE INDEX ft_mint_testnet_idx_owner_id ON ft_mint_testnet(owner_id);
CREATE INDEX ft_mint_testnet_idx_token_id ON ft_mint_testnet(token_id);
CREATE INDEX ft_mint_testnet_idx_id ON ft_mint_testnet(id);
CREATE INDEX ft_mint_testnet_idx_id_owner_id ON ft_mint_testnet(id, owner_id);
CREATE INDEX ft_mint_testnet_idx_id_token_id ON ft_mint_testnet(id, token_id);
CREATE INDEX ft_mint_testnet_idx_timestamp_owner_id ON ft_mint_testnet(timestamp, owner_id);
CREATE INDEX ft_mint_testnet_idx_timestamp_token_id ON ft_mint_testnet(timestamp, token_id);
CREATE INDEX ft_mint_testnet_idx_token_id_amount ON ft_mint_testnet(token_id, amount);
CREATE INDEX ft_mint_testnet_idx_id_token_id_amount ON ft_mint_testnet(id, token_id, amount);
CREATE INDEX ft_mint_testnet_idx_timestamp_token_id_amount ON ft_mint_testnet(timestamp, token_id, amount);

CREATE TABLE ft_mint (
    id SERIAL,
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

CREATE INDEX ft_mint_idx_owner_id ON ft_mint(owner_id);
CREATE INDEX ft_mint_idx_token_id ON ft_mint(token_id);
CREATE INDEX ft_mint_idx_id ON ft_mint(id);
CREATE INDEX ft_mint_idx_id_owner_id ON ft_mint(id, owner_id);
CREATE INDEX ft_mint_idx_id_token_id ON ft_mint(id, token_id);
CREATE INDEX ft_mint_idx_timestamp_owner_id ON ft_mint(timestamp, owner_id);
CREATE INDEX ft_mint_idx_timestamp_token_id ON ft_mint(timestamp, token_id);
CREATE INDEX ft_mint_idx_token_id_amount ON ft_mint(token_id, amount);
CREATE INDEX ft_mint_idx_id_token_id_amount ON ft_mint(id, token_id, amount);
CREATE INDEX ft_mint_idx_timestamp_token_id_amount ON ft_mint(timestamp, token_id, amount);


CREATE TABLE ft_burn_testnet (
    id SERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    transaction_id TEXT NOT NULL,
    receipt_id TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    token_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    memo TEXT
);

SELECT create_hypertable('ft_burn_testnet', 'timestamp');

CREATE INDEX ft_burn_testnet_idx_owner_id ON ft_burn_testnet(owner_id);
CREATE INDEX ft_burn_testnet_idx_token_id ON ft_burn_testnet(token_id);
CREATE INDEX ft_burn_testnet_idx_id ON ft_burn_testnet(id);
CREATE INDEX ft_burn_testnet_idx_id_owner_id ON ft_burn_testnet(id, owner_id);
CREATE INDEX ft_burn_testnet_idx_id_token_id ON ft_burn_testnet(id, token_id);
CREATE INDEX ft_burn_testnet_idx_timestamp_owner_id ON ft_burn_testnet(timestamp, owner_id);
CREATE INDEX ft_burn_testnet_idx_timestamp_token_id ON ft_burn_testnet(timestamp, token_id);
CREATE INDEX ft_burn_testnet_idx_token_id_amount ON ft_burn_testnet(token_id, amount);
CREATE INDEX ft_burn_testnet_idx_id_token_id_amount ON ft_burn_testnet(id, token_id, amount);
CREATE INDEX ft_burn_testnet_idx_timestamp_token_id_amount ON ft_burn_testnet(timestamp, token_id, amount);

CREATE TABLE ft_burn (
    id SERIAL,
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

CREATE INDEX ft_burn_idx_owner_id ON ft_burn(owner_id);
CREATE INDEX ft_burn_idx_token_id ON ft_burn(token_id);
CREATE INDEX ft_burn_idx_id ON ft_burn(id);
CREATE INDEX ft_burn_idx_id_owner_id ON ft_burn(id, owner_id);
CREATE INDEX ft_burn_idx_id_token_id ON ft_burn(id, token_id);
CREATE INDEX ft_burn_idx_timestamp_owner_id ON ft_burn(timestamp, owner_id);
CREATE INDEX ft_burn_idx_timestamp_token_id ON ft_burn(timestamp, token_id);
CREATE INDEX ft_burn_idx_token_id_amount ON ft_burn(token_id, amount);
CREATE INDEX ft_burn_idx_id_token_id_amount ON ft_burn(id, token_id, amount);
CREATE INDEX ft_burn_idx_timestamp_token_id_amount ON ft_burn(timestamp, token_id, amount);
