import { query, sql } from "@example/db";
import type { SQLStatement } from "sql-template-strings";
const gate = runtimeFlag;
const pure = sql`SELECT 1`;
function withOffset(input: SQLStatement): SQLStatement {
  return input.append(sql` OFFSET 99`);
}
function wrapper(pure: SQLStatement): SQLStatement {
  return withOffset(pure);
}
// Walking the wrapper's parameter must not invalidate the global namesake.
query(gate ? pure : sql`SELECT 2`);
