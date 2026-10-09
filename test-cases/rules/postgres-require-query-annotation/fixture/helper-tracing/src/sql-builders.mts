import sql, { type SQLStatement } from "sql-template-strings";

export function importedOrdersSql(): SQLStatement {
  return sql``.append("SELECT id FROM imported_orders WHERE shipped_at IS NULL");
}
