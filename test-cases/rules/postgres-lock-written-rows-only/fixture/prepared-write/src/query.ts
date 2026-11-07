import { beginTransaction } from "@example/db";
export async function preparedIsNotExecuted(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`PREPARE later_update AS UPDATE items SET value = 1 WHERE id = $1`);
}
