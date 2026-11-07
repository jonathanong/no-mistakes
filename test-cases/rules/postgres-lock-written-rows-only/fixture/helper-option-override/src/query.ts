import { beginTransaction, type TransactionQuery } from "@example/db";
export async function writeUser(query: TransactionQuery, userId: string, alternate: TransactionQuery) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [userId], { executor: alternate });
}
export async function updateUser(userId: string) {
  const tx = await beginTransaction();
  const other = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [userId]);
  await writeUser(tx.query, userId, other.query);
}
