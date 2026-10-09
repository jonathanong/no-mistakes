import sql, { type SQLStatement } from "sql-template-strings";

export function annotatedOrdersSql(): SQLStatement {
  return sql`/* namespaced */ SELECT id FROM orders`;
}
