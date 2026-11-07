import { beginTransaction } from "@example/db";
export async function separateTransactions(id: string) {
  const first = await beginTransaction();
  const second = await beginTransaction();
  await first.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await second.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
