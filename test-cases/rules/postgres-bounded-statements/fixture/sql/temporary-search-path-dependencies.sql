-- The view's source is ambiguous during CREATE, but a later definite CASCADE
-- must retire its temporary identity before the catalog namesake is read.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = missing_schema, pg_temp, public;
CREATE TEMP VIEW orders AS SELECT * FROM accounts;
RESET search_path;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;

-- Hiding the outer temporary source must retain the executed IN subquery.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = missing_schema, pg_temp, public;
SELECT * FROM accounts WHERE id IN (SELECT id FROM orders);
