-- LOCAL restoration must restore the complete physical-path proof as well.
SET search_path = public, pg_temp;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
BEGIN;
SET LOCAL search_path = other, pg_temp;
COMMIT;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
BEGIN;
SET LOCAL search_path = other, pg_temp;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
ROLLBACK;
SELECT * FROM orders;
RESET search_path;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
