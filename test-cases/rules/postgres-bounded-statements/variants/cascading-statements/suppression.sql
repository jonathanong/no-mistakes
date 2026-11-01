SELECT a.id FROM accounts a
-- no-mistakes-disable-next-line postgres-bounded-statements
JOIN orders o ON o.account_id = a.id;
SELECT a.id FROM accounts a JOIN orders o ON o.account_id = a.id; SELECT id FROM invoices;
