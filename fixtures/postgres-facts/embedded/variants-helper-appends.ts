import { sql } from "@example/db"; import { type SQLStatement } from "sql-template-strings"; /* Configured tags splice fragments; sql-template-strings binds them. */
import { query } from "@example/db";
const gate = runtimeFlag;
function withStaticAppends(statement: SQLStatement): SQLStatement {
  statement.append(" FROM us\u0065rs");
  statement.append(` WHERE active = true`);
  statement.append(String.raw` AND name = E'\\n'`);
  return statement.append(" LIMIT 5");
}
query(gate ? withStaticAppends(sql`SELECT id`) : sql`SELECT 2`);
