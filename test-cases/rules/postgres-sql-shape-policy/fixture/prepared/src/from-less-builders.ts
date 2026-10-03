import sql from "sql-template-strings";

// A complete query without a relation still owns LIMIT facts.
export function selectPage() {
  return sql`SELECT 1
LIMIT 500`;
}
export function valuesPage() {
  return sql`VALUES (1)
LIMIT 500`;
}

export function suppressedSelect() {
  return sql`SELECT 1
-- no-mistakes-disable-next-line postgres-sql-shape-policy
LIMIT 500`;
}
export function suppressedValues() {
  return sql`VALUES (1)
LIMIT 500`; // no-mistakes-disable-line postgres-sql-shape-policy
}
