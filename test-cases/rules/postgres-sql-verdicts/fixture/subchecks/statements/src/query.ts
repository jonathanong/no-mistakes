import { query } from "@example/db";
// Each call keeps sibling obligation outcomes independent.
query("CREATE TABLE items (id integer)");
query("ALTER TABLE items ADD COLUMN note text");
query("TRUNCATE items");
query("SET work_mem = '4MB'");
query("SELECT set_config('app.guard', 'off', true)");
query("SELECT 1");
