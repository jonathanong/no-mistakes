-- A false WHERE conjunct bounds every statement at zero rows.
UPDATE accounts SET name = 'x' WHERE FALSE;
DELETE FROM accounts WHERE (FALSE);
SELECT 1 FROM accounts WHERE TRUE AND FALSE;
UPDATE accounts SET name = 'x' WHERE FALSE AND name = $1;
DELETE FROM accounts WHERE name = $1 AND (FALSE);
-- A false disjunct does not make the entire predicate false.
UPDATE accounts SET name = 'x' WHERE FALSE OR TRUE;
DELETE FROM accounts WHERE TRUE;
SELECT 1 FROM accounts WHERE TRUE AND name = $1;
