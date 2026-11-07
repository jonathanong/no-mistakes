import { beginTransaction, type TransactionQuery } from "@example/db";
export async function lockUser(query: TransactionQuery, userId: string) {
  await query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [userId]);
}
export async function updateMembership(userId: string) {
  const tx = await beginTransaction();
  await lockUser(tx.query, userId);
  await tx.query(`UPDATE memberships SET value = 1 WHERE user_id = $1`, [userId]);
}
