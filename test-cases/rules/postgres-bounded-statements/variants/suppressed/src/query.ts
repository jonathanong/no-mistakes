import { query, sql } from "@example/db";
query(flag
  // no-mistakes-disable-next-line postgres-bounded-statements
  ? sql`SELECT id FROM accounts`
  : sql`SELECT id FROM accounts LIMIT 1`);
