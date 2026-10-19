import { query, sql } from "@example/db";

export function load(accountId: string) {
  return query(sql`
    SELECT id FROM documents WHERE account_id = ${accountId}
  `);
}

export function shadowed(
  sql: (strings: TemplateStringsArray, ...values: unknown[]) => string,
  accountId: string,
) {
  return query(sql`
    SELECT id FROM documents WHERE account_id = ${accountId}
  `);
}
