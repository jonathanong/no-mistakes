import { beginTransaction, type TransactionQuery } from "@example/db";
export async function writeUser(query: TransactionQuery, userId: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [userId]);
}
export async function updateUser(userId: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [userId]);
  return;
  await writeUser(tx.query, userId);
}
