import { query, sql } from "@example/db";
import type { SQLStatement } from "sql-template-strings";
function fill(input: SQLStatement): SQLStatement {
  return input.append(sql`SELECT pg_sleep(1)`);
}
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(flag ? fill(sql``) : sql`SELECT 1`);
