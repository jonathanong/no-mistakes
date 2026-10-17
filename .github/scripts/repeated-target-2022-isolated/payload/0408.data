import { query, sql } from "@example/db";
const gate = runtimeFlag;
query(gate ? sql`
  SELECT pg_catalog.set_config(E'statement_timeout', '1s', true),
         set_config($setting$lock_timeout$setting$, '2s', true),
         tools.set_config('search_path', 'custom', false),
         set_config(setting_name, '3s', false),
         set_config(1, '4s', false);
  SELECT set_config;
  SELECT set_config();
  SELECT "SET_CONFIG"('lock_timeout', 'custom', false), "PG_CATALOG".set_config('lock_timeout', 'catalog_custom', false), "set_config"('lock_timeout', 'actual', false);
  SET LOCAL app.trace_id TO 'trace';
  ALTER DATABASE app SET statement_timeout = '5s';
  ALTER SYSTEM SET lock_timeout = '6s';
` : sql`SELECT 1`);
