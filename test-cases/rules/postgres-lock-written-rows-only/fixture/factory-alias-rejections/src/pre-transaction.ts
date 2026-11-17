import { beginTransaction } from "@example/db";
export async function rejected(id: string) {
  const q = tx;
  const tx = await beginTransaction();
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
