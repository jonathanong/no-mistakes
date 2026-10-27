import { query } from "@example/db";

export function reversed(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id) SELECT $1::uuid, input.user_id FROM unnest($2::uuid[]) AS input(user_id) ORDER BY input.user_id DESC ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountId, userIds],
  );
}
