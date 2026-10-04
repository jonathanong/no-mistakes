-- Permanent accounts.id must not prove ownership after a differently shaped temporary shadow.
CREATE TEMP TABLE accounts (temp_only uuid);
SELECT * FROM orders o WHERE o.id IN (SELECT id FROM accounts LIMIT 1);
SELECT * FROM orders o WHERE o.id IN (SELECT id FROM public.accounts LIMIT 1);
SELECT * FROM orders o WHERE o.id IN (SELECT a.temp_only FROM accounts a LIMIT 1);
DROP TABLE accounts;
SELECT * FROM orders o WHERE o.id IN (SELECT id FROM accounts LIMIT 1);
CREATE TEMP TABLE accounts (temp_only uuid);
SELECT * FROM orders o CROSS JOIN LATERAL (SELECT id FROM accounts LIMIT 1) a WHERE o.id = a.id;
SELECT * FROM orders o CROSS JOIN LATERAL (SELECT id FROM public.accounts LIMIT 1) a WHERE o.id = a.id;
DROP TABLE accounts;
SELECT * FROM orders o CROSS JOIN LATERAL (SELECT id FROM accounts LIMIT 1) a WHERE o.id = a.id;
