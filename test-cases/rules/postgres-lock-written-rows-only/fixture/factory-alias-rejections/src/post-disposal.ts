import { beginTransaction } from "@example/db";
export async function rejected(id: string) {
  let q;
  {
    await using tx = await beginTransaction();
    q = tx;
  }
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await q(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
