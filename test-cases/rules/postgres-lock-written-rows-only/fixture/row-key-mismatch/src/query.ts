import { beginTransaction } from "@example/db";
export async function updateDifferentRow(lockId: string, writeId: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR NO KEY UPDATE`, [lockId]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [writeId]);
}
