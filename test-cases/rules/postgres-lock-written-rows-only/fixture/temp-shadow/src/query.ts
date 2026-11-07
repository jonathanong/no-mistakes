import { beginTransaction } from "@example/db";
export async function shadowRelation() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("CREATE TEMP TABLE items (id integer, value integer)");
  await tx.query("UPDATE items SET value = 2 WHERE id = 1");
}
