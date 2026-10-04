-- Repeated caller-sized keys remain finite even beside an independent SRF.
UPDATE accounts a SET name = 'x' FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry) s WHERE a.id = s.id;
UPDATE accounts a SET name = 'x' FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry) s(key, entry) WHERE a.id = s.key;
WITH s AS (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry) UPDATE accounts a SET name = 'x' FROM s WHERE a.id = s.id;
WITH s(key, entry) AS (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry) UPDATE accounts a SET name = 'x' FROM s WHERE a.id = s.key;
UPDATE accounts a SET name = 'x' FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry UNION ALL SELECT $2::uuid AS key, pg_ls_dir('.')) s WHERE a.id = s.id;
-- SRF-derived, unknown, and row-dependent keys retain their original opacity.
UPDATE accounts a SET name = 'x' FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry) s WHERE a.id = s.entry::uuid;
UPDATE accounts a SET name = 'x' FROM (SELECT custom_srf($1) AS id, pg_ls_dir('.') AS entry) s WHERE a.id = s.id;
UPDATE accounts a SET name = 'x' FROM (SELECT id, pg_ls_dir('.') AS entry FROM orders) s WHERE a.id = s.id;
-- UNION output identity follows ordinal positions, not right-arm alias spelling.
UPDATE accounts a SET name = 'x' FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry UNION ALL SELECT pg_ls_dir('.')::uuid AS entry, $2::uuid AS id) s WHERE a.id = s.id;
UPDATE accounts a SET name = 'x' FROM (SELECT $1::app.uuid AS id, pg_ls_dir('.') AS entry) s WHERE a.id = s.id;
UPDATE accounts a SET name = 'x' FROM (SELECT (($1::uuid)), pg_ls_dir('.')) s(id, entry) WHERE a.id = s.id;
-- Wildcard output width cannot supply an ordinal caller-value proof.
UPDATE accounts a SET name = 'x' FROM (SELECT o.*, $1::uuid AS key, pg_ls_dir('.') AS entry FROM orders o) s WHERE a.id = s.key;
UPDATE accounts a SET name = 'x' FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry UNION ALL SELECT o.* FROM (SELECT id, account_id FROM orders) o) s WHERE a.id = s.id;
