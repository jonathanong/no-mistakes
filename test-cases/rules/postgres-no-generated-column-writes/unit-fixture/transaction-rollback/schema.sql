CREATE TABLE orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
-- A rollback must restore the protected table, including nested dynamic DDL.
BEGIN;
DO $$ BEGIN EXECUTE 'DROP TABLE orders'; END $$;
ROLLBACK;
BEGIN;
ALTER TABLE orders ADD COLUMN rolled_back int GENERATED ALWAYS AS (id + 2) STORED;
ROLLBACK;
BEGIN;
SAVEPOINT keep;
ALTER TABLE orders ADD COLUMN removed_at_savepoint int GENERATED ALWAYS AS (id + 3) STORED;
ROLLBACK TO SAVEPOINT keep;
ALTER TABLE orders ADD COLUMN committed int GENERATED ALWAYS AS (id + 4) STORED;
RELEASE SAVEPOINT keep;
COMMIT;
