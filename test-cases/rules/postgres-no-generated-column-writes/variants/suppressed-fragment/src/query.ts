import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-no-generated-column-writes
  UPDATE items SET created_at = now() WHERE id = $1` : sql`UPDATE items SET note = $2 WHERE id = $1`}`);
