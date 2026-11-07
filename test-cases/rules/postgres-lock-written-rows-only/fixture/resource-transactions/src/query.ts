import { beginTransaction } from "@example/db";
export async function written(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function unwritten(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT id FROM parents WHERE id = $1 FOR KEY SHARE`, [id]);
  await tx.query(`INSERT INTO children (parent_id) VALUES ($1)`, [id]);
}
export function synchronousResource(id: string) {
  using tx = beginTransaction();
  tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
