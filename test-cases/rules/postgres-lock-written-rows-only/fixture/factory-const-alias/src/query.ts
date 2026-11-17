import { beginTransaction } from "@example/db";
export async function unwritten(id: string) {
  await using transaction = await beginTransaction();
  const query = transaction;
  await query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await transaction.commit();
}
export async function chainedWritten(id: string) {
  const transaction = await beginTransaction();
  const execute = transaction;
  const renamed = execute;
  await renamed.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await transaction.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
  await transaction.commit();
}
