-- A false WHERE conjunct bounds every statement at zero rows.
UPDATE accounts SET name = 'x' WHERE FALSE;
DELETE FROM accounts WHERE (FALSE);
SELECT 1 FROM accounts WHERE TRUE AND FALSE;
UPDATE accounts SET name = 'x' WHERE FALSE AND name = $1;
DELETE FROM accounts WHERE name = $1 AND (FALSE);
UPDATE accounts SET name = 'x' FROM orders WHERE FALSE;
DELETE FROM accounts USING orders WHERE FALSE;
SELECT 1 FROM orders WHERE false HAVING true ORDER BY generate_series(1, 1000000);
-- A false WHERE predicate still caps an ORDER BY SRF when no implicit group exists.
SELECT 1 FROM orders WHERE false ORDER BY generate_series(1, 1000000);
-- A false disjunct does not make the entire predicate false.
UPDATE accounts SET name = 'x' WHERE FALSE OR TRUE;
DELETE FROM accounts WHERE TRUE;
SELECT 1 FROM accounts WHERE TRUE AND name = $1;
-- A true WHERE predicate leaves the expanding query unbounded.
SELECT 1 FROM orders WHERE true ORDER BY generate_series(1, 1000000);
