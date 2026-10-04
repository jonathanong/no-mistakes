-- g exposes n, so the bare id reads the outer account and cannot pin it.
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM generate_series(1, 1) AS g(n) LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM generate_series(1, 1) AS g LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM (SELECT code FROM currencies) c LIMIT 1);
WITH c AS (SELECT code FROM currencies) DELETE FROM accounts a WHERE a.id IN (SELECT id FROM c LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (WITH c AS (SELECT code FROM currencies) SELECT id FROM c LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM (SELECT id FROM orders) c LIMIT 1);
WITH c AS (SELECT id FROM orders) DELETE FROM accounts a WHERE a.id IN (SELECT id FROM c LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM (SELECT * FROM unknown_relation) c LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM unknown_function() f LIMIT 1);
