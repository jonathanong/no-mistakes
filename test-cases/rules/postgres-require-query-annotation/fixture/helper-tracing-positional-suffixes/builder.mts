import sql from "sql-template-strings";

export function validSql() {
  return sql`SELECT $${+10}, ${"value"}`;
}

export function invalidSql() {
  return sql`SELECT $${-1.5}, ${"value"}`;
}
