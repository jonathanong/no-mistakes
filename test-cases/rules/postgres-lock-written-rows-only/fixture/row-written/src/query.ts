import { beginTransaction } from "@example/db";
export async function updateRow(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function deleteRow(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR KEY SHARE`, [id]);
  await tx.query(`DELETE FROM items WHERE id = $1`, [id]);
}
