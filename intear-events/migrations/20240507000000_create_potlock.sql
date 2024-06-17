BEGIN;

CREATE TABLE potlock_donation (
    id SERIAL,
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

CREATE INDEX potlock_donation_idx_donor_id ON potlock_donation(donor_id);
CREATE INDEX potlock_donation_idx_project_id ON potlock_donation(project_id);
CREATE INDEX potlock_donation_idx_referrer_id ON potlock_donation(referrer_id);

CREATE TABLE potlock_pot_project_donation (
    id SERIAL,
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

CREATE INDEX potlock_pot_project_donation_idx_pot_id ON potlock_pot_project_donation(pot_id);
CREATE INDEX potlock_pot_project_donation_idx_donor_id ON potlock_pot_project_donation(donor_id);
CREATE INDEX potlock_pot_project_donation_idx_project_id ON potlock_pot_project_donation(project_id);
CREATE INDEX potlock_pot_project_donation_idx_referrer_id ON potlock_pot_project_donation(referrer_id);

CREATE TABLE potlock_pot_donation (
    id SERIAL,
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
    chef_fee NUMERIC
);

SELECT create_hypertable('potlock_pot_donation', 'timestamp');

CREATE INDEX potlock_pot_donation_idx_pot_id ON potlock_pot_donation(pot_id);
CREATE INDEX potlock_pot_donation_idx_donor_id ON potlock_pot_donation(donor_id);
CREATE INDEX potlock_pot_donation_idx_referrer_id ON potlock_pot_donation(referrer_id);

COMMIT;