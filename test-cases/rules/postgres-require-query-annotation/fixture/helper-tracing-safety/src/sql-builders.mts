import sql, { type SQLStatement } from "sql-template-strings";

export function reexportedOrdersSql(): SQLStatement {
  return sql`/* imported */ SELECT id FROM orders`;
}
