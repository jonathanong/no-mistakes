import { beginTransaction } from "@example/db";
export async function accepted(id: string) {
  await using transaction = await beginTransaction();
  const query = transaction;
  await query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await transaction(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
