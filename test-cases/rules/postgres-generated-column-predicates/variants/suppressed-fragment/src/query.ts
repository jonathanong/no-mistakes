import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-generated-column-predicates
  SELECT id FROM orders WHERE created_at > $1` : sql`SELECT id FROM orders WHERE id > $1`}`);
