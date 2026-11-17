import { beginTransaction, query as pool } from "@example/db";
export async function rejected(id: string) {
  await using tx = await beginTransaction();
  let q = tx;
  q = pool;
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
}
