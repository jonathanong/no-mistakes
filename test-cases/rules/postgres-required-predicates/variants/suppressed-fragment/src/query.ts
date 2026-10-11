import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-required-predicates
  SELECT id FROM accounts` : sql`SELECT id FROM accounts WHERE active IS TRUE ORDER BY id`}`);
