import sql, { type SQLStatement } from "sql-template-strings";

export default function buildDefaultFunction(): SQLStatement {
  return sql`/* defaultFunction */ SELECT id FROM orders`;
}
