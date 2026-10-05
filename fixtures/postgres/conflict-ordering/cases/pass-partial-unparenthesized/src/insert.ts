import { query } from "@example/db";

// The catalog stores `((deleted_at IS NULL) AND (user_id IS NOT NULL))`; the writer omits the
// redundant parentheses and must still resolve to the same partial index.
export function addMembers(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO memberships (account_id, user_id)
     SELECT $1::uuid AS account_id, input.user_id
     FROM unnest($2::uuid[]) AS input(user_id)
     ORDER BY account_id, input.user_id
     ON CONFLICT (account_id, user_id) WHERE deleted_at IS NULL AND user_id IS NOT NULL
     DO NOTHING`,
    [accountId, userIds],
  );
}
