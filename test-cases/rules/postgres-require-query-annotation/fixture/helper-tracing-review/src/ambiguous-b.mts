import sql, { type SQLStatement } from "sql-template-strings";
export function ambiguousSql(): SQLStatement {
  return sql`/* from b */ SELECT 2`;
}
