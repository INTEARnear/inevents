BEGIN;

ALTER TABLE price_pool DROP COLUMN amount_in;
ALTER TABLE price_pool DROP COLUMN amount_out;
ALTER TABLE price_pool ADD COLUMN block_height BIGINT NOT NULL;
ALTER TABLE price_token ADD COLUMN block_height BIGINT NOT NULL;

COMMIT;
