import { query } from "@example/db";

export function constants(accountId: string, userId: string) {
  return query(
    `INSERT INTO members (account_id, user_id) SELECT $1::uuid, $2::uuid FROM generate_series(1, 3) ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountId, userId],
  );
}
