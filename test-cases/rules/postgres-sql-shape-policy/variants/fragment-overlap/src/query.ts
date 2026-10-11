import sql, { type SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
function withSleep(statement: SQLStatement): SQLStatement {
  return statement.append(sql` WHERE pg_sleep(1) IS NULL`);
}
query(flag ? withSleep(sql`SELECT 1`) : sql`SELECT 2`);
const branching = sql`SELECT 3`;
if (flag) branching.append(sql` WHERE pg_sleep(1) IS NULL`);
query(branching);
const unrelated = sql`SELECT 4`;
// The two separate tokens share a physical line and identical diagnostic JSON.
unrelated.append(sql` WHERE pg_sleep(1) IS NULL`); query(flag ? sql`SELECT pg_sleep(1)` : sql`SELECT 5`);
