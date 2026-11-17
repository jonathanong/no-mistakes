import { beginTransaction } from "@example/db";
export async function rejected(id: string) {
  const tx = await beginTransaction();
  const holder = { query: tx };
  const { query } = holder;
  await query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
