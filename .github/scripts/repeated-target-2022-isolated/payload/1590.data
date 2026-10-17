import { query, sql } from "@example/db";

// The ternary picks one of two SQL fragments at runtime; neither branch is a
// bind value, so the composed statement must classify as Dynamic.
export function load(accountId: string, includeDeleted: boolean) {
  return query(sql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId}
    ${includeDeleted ? sql`` : sql`AND deleted_at IS NULL`}
  `);
}
