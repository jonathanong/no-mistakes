import { beginTransaction } from "@example/db";
export async function shadowedRelation(id: string) {
  const tx = await beginTransaction();
  await tx.query(`WITH items AS (SELECT id FROM parents) SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
