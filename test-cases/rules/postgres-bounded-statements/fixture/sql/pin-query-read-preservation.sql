-- Correlation invalidates key proof but must not discard the executed inner read.
SELECT * FROM accounts a WHERE a.id IN (SELECT o.id FROM orders o WHERE o.account_id = a.id);
SELECT * FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.id FROM orders o WHERE o.account_id = a.id);
SELECT * FROM accounts a WHERE a.id IN (SELECT o.id FROM orders o WHERE o.id = $1 AND o.account_id = a.id);
SELECT * FROM accounts a WHERE a.id = ANY(SELECT o.id FROM orders o WHERE o.account_id = a.id);
SELECT * FROM accounts a WHERE a.id IN (SELECT o.id FROM orders o WHERE o.id = $1);
-- A temporary outer source has no permanent key credit; inner permanent reads still execute.
CREATE TEMP TABLE accounts(id uuid);
SELECT * FROM accounts WHERE id IN (SELECT id FROM orders);
CREATE TEMP VIEW token_view AS SELECT 1 AS id;
SELECT * FROM token_view WHERE id IN (SELECT id FROM orders);
CREATE TEMP TABLE orders(id uuid, account_id uuid);
SELECT * FROM accounts a WHERE a.id IN (SELECT o.id FROM orders o WHERE o.account_id = a.id);
DROP TABLE orders;
DROP TABLE accounts;
-- no-mistakes-disable-next-line postgres-bounded-statements
SELECT * FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.id FROM orders o WHERE o.account_id = a.id);
