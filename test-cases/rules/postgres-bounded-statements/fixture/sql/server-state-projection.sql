-- Literal arguments and empty argument lists do not bound server-state SRFs.
DELETE FROM accounts WHERE id IN (SELECT pg_ls_dir('.')::uuid);
DELETE FROM accounts WHERE id IN (SELECT pg_catalog.pg_get_keywords()::text::uuid);
DELETE FROM accounts WHERE id IN (SELECT ts_stat('SELECT vector FROM documents')::text::uuid);
-- Explicit result caps and known caller-sized SRFs retain their bounded evidence.
DELETE FROM accounts WHERE id IN (SELECT pg_ls_dir('.')::uuid LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT unnest($1::uuid[]));
DELETE FROM accounts WHERE id IN (SELECT generate_series($1, 10)::text::uuid);
DELETE FROM accounts WHERE id IN (SELECT pg_catalog.generate_subscripts($1, 1)::text::uuid);
-- This catalog function has exactly one output row, independent of server state.
DELETE FROM accounts WHERE id IN (SELECT pg_stat_get_recovery_prefetch()::text::uuid);
