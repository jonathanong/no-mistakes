SELECT set_config(E'session_replication_role', 'replica', true);
SELECT "pg_catalog"."set_config"($name$session_replication_role$name$, 'replica', true);
SELECT set_config(42, 'replica', true);
-- Incomplete utility syntax retains recoverable named settings, without panic.
SET app.;
SET 42;
SET;
