import { query, sql } from "@example/db";

// An unconditional nested fragment is still spliced SQL, not a bind value.
export function load(accountId: string) {
  return query(sql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId} ${sql`AND deleted_at IS NULL`}
  `);
}
