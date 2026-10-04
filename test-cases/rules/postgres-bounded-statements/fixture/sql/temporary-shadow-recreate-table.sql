-- A bare CREATE restores the physical shadow after the first DROP. The second
-- DROP must not retire the underlying temporary table on stale snapshot evidence.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = real_schema, pg_temp, public;
DROP TABLE accounts;
CREATE TABLE accounts(id uuid);
DROP TABLE accounts;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
