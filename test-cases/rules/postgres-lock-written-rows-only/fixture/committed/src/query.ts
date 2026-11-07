import { beginTransaction } from "@example/db";
export async function afterCommit(id: string) {
  const tx = await beginTransaction();
  await tx.query(`COMMIT`);
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
