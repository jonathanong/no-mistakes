import sql, { type SQLStatement } from "sql-template-strings";

export function annotatedOrdersSql(): SQLStatement {
  return sql`/* importedOrders */ SELECT id FROM orders`;
}

export function unannotatedOrdersSql(): SQLStatement {
  return sql`SELECT id FROM orders`;
}

export function defaultFunctionSql(): SQLStatement {
  return sql`/* defaultFunction */ SELECT id FROM orders`;
}
