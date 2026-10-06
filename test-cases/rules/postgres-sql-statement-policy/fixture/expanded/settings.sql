SET session_replication_role = replica;
SET LOCAL SESSION_REPLICATION_ROLE TO replica;
SET SESSION session_replication_role = origin;
SELECT set_config('session_replication_role', 'replica', true);
SELECT pg_catalog.set_config('SESSION_REPLICATION_ROLE', 'origin', false);
ALTER DATABASE app SET session_replication_role = replica;
ALTER SYSTEM SET session_replication_role = origin;
SET app.guard = 'disabled';
SELECT set_config('app.guard', 'enabled', false);
-- Inert text and same-spelled non-builtin functions are not setting changes.
SELECT 'SET session_replication_role = replica; set_config(''session_replication_role'', ''replica'', true)';
SELECT other.set_config('session_replication_role', 'replica', true);
SELECT set_config(setting_name, 'replica', true);
SELECT set_config('session_replication_role' || suffix, 'replica', true);
SELECT "SET_CONFIG"('session_replication_role', 'replica', true);
SET statement_timeout = '1s';
