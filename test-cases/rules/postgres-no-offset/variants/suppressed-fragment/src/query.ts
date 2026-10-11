import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-no-offset
  SELECT id FROM accounts OFFSET 1` : sql`SELECT id FROM accounts LIMIT 1`}`);
