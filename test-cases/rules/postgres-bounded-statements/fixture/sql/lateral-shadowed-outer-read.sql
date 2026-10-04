-- The LATERAL output named id must not erase its own read of the mutation row.
DELETE FROM accounts WHERE id IN (SELECT id FROM LATERAL (SELECT id) d LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT id FROM LATERAL (SELECT a.id) d LIMIT 1);
-- Only preceding sources are visible to a LATERAL query.
DELETE FROM accounts WHERE id IN (SELECT d.id FROM currencies c, LATERAL (SELECT id) d LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT d.id FROM (SELECT 7 AS id) c, LATERAL (SELECT id) d LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT d.id FROM LATERAL (SELECT id) d, (SELECT 7 AS id) c LIMIT 1);
-- Scalar subqueries can still resolve against all sources in their containing query.
DELETE FROM accounts WHERE id IN (SELECT (SELECT d.id) FROM (SELECT 7 AS id) d LIMIT 1);
-- A non-LATERAL derived query also cannot read its own derived output.
DELETE FROM accounts WHERE id IN (SELECT id FROM (SELECT id) d LIMIT 1);
