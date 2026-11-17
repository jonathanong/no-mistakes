import { beginTransaction } from "@example/db";
export async function accepted(id: string) {
  await using tx = await beginTransaction();
  const q = tx;
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
