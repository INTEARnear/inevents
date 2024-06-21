BEGIN;

ALTER TABLE price_token DROP COLUMN price_near;

COMMIT;
