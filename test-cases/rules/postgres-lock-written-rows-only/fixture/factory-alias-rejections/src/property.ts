import { beginTransaction } from "@example/db";
export async function rejected(id: string) {
  const tx = await beginTransaction();
  const holder = { tx };
  const q = holder.tx;
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
