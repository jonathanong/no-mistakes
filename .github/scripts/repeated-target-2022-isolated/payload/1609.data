import { query, sql } from "@example/db";

const status = "active";

// Plain values stay parameterized binds, so the statement is still Inline.
// `label` is a String.raw result (a string value), not a SQL fragment.
export function load(accountId: string, ids: string[]) {
  const label = String.raw`a\b`;
  return query(sql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId}
      AND status = ${status}
      AND id = ANY(${ids})
      AND label = ${label}
      AND kind = ${accountId ? "a" : "b"}
  `);
}
