import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-lock-ordering
  SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE` : sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE SKIP LOCKED`}`);
