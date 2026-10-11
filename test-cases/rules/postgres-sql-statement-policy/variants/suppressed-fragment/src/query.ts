import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-sql-statement-policy
  CREATE INDEX accounts_email ON accounts (email)` : sql`SELECT 1`}`);
