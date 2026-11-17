import { beginTransaction } from "@example/db";
export async function rejected(id: string) {
  const tx = await beginTransaction();
  const q = tx;
  await tx.commit();
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await q(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
