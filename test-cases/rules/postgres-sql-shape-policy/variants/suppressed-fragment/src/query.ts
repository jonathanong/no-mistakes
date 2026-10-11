import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-sql-shape-policy
  SELECT pg_sleep(1)` : sql`SELECT 1`}`);
