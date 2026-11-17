import { beginTransaction } from "@example/db";
export async function reassigned(id: string) {
  await using transaction = await beginTransaction();
  let query = transaction;
  query = await beginTransaction();
  await query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await transaction.commit();
}
