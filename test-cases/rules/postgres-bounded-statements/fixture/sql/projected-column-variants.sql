-- Explicit aliases and first-arm labels define source outputs; limits size their rows.
DELETE FROM accounts WHERE id IN (SELECT id FROM (SELECT code FROM currencies) c(id) LIMIT 1);
WITH c(id) AS (SELECT code FROM currencies) DELETE FROM accounts WHERE id IN (SELECT id FROM c LIMIT 1);
DELETE FROM accounts WHERE id IN (WITH c(id) AS (SELECT code FROM currencies) SELECT id FROM c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM (SELECT o.id FROM orders o) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM (SELECT code AS id FROM currencies UNION ALL SELECT id FROM orders) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM ((SELECT id FROM orders)) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM (VALUES (1)) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM generate_series(1, 3) id LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[]) c(id) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM app.generate_series(1, 3) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM (SELECT id FROM orders) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM (orders o JOIN currencies c ON o.status = c.code) LIMIT 1);
