import type { TransactionQuery } from "@example/db";
export async function writeUser(query: TransactionQuery, userId: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [userId]);
}
