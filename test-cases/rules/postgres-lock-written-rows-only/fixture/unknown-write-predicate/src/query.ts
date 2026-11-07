import { beginTransaction } from "@example/db";
export async function updateFrom(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = other.value FROM other WHERE items.id = $1 AND other.id = items.id`, [id]);
}
export async function deleteUsing(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`DELETE FROM items USING other WHERE items.id = $1 AND other.id = items.id`, [id]);
}
