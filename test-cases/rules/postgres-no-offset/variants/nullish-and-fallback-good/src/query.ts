import { query, sql } from "@example/db";
const gate = runtimeFlag;
query(
  ((gate ? null : sql`SELECT id FROM accounts LIMIT 1`)
    && sql`SELECT id FROM accounts LIMIT 2`)
    ?? sql`SELECT id FROM accounts LIMIT 3`
);
