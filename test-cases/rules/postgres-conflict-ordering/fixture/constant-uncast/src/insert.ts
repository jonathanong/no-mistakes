import { query } from "@example/db";

export function uncast(accountId: string, userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id) SELECT $1, input.user_id FROM unnest($2::uuid[]) AS input(user_id) ORDER BY $1::uuid, 2 ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountId, userIds],
  );
}
