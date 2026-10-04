-- Independently bounded output must not erase reads needed to compute its pins.
SELECT * FROM (SELECT 1 FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.account_id FROM orders o)) AS d;
WITH d AS (SELECT 1 FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.account_id FROM orders o)) SELECT * FROM d;
SELECT 1 FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.account_id FROM orders o) UNION ALL SELECT 1;
-- Capping the pin query itself proves its underlying reads bounded.
SELECT * FROM (SELECT 1 FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.account_id FROM orders o LIMIT 1)) AS d;
WITH d AS (SELECT 1 FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.account_id FROM orders o LIMIT 1)) SELECT * FROM d;
SELECT 1 FROM accounts a WHERE a.id = $1 AND a.id IN (SELECT o.account_id FROM orders o LIMIT 1) UNION ALL SELECT 1;
