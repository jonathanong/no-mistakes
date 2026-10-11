import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-bounded-statements
  SELECT id FROM accounts` : sql`SELECT id FROM accounts LIMIT 2`}`);
