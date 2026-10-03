-- Other statement projections must be identical when row-bound demand is additive.
SELECT id FROM accounts WHERE id = $1;
UPDATE accounts SET name = $1 WHERE id = $2;
DELETE FROM accounts WHERE id = $1;
INSERT INTO accounts (id, name) VALUES ($1, $2);
