import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-explicit-columns
  SELECT * FROM accounts` : sql`SELECT id FROM accounts`}`);
