import sql, { type SQLStatement } from "sql-template-strings";
export function ambiguousSql(): SQLStatement {
  return sql`/* from a */ SELECT 1`;
}
