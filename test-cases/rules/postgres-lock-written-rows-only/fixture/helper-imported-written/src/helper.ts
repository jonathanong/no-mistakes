import type { TransactionQuery } from "@example/db";
export async function lockUser(query: TransactionQuery, userId: string) {
  await query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [userId]);
}
