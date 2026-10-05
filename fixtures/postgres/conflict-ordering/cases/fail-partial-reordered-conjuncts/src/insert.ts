import { query } from "@example/db";

// Conjunct order stays significant: logical equivalence is deliberately not guessed, so a
// reordered predicate remains an unresolved arbiter.
export function addMembers(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO memberships (account_id, user_id)
     SELECT $1::uuid AS account_id, input.user_id
     FROM unnest($2::uuid[]) AS input(user_id)
     ORDER BY account_id, input.user_id
     ON CONFLICT (account_id, user_id) WHERE user_id IS NOT NULL AND deleted_at IS NULL
     DO NOTHING`,
    [accountId, userIds],
  );
}
