import { beginTransaction, query as pool } from "@example/db";
export async function rejected(id: string, condition: boolean) {
  await using tx = await beginTransaction();
  const q = condition ? tx : pool;
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
}
