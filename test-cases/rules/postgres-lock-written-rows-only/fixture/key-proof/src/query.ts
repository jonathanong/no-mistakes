import { beginTransaction } from "@example/db";
export async function literalSame() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("UPDATE items SET value = 2 WHERE id = 1");
}
export async function literalDifferent() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("UPDATE items SET value = 2 WHERE id = 2");
}
export async function reversedCastAndQualifiedAlias() {
  const tx = await beginTransaction();
  await tx.query("SELECT i.id FROM items AS i WHERE CAST(1 AS integer) = i.id FOR UPDATE");
  await tx.query("UPDATE items AS i SET value = 2 WHERE i.id = 1");
}
export async function wholeRelationWrite() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("UPDATE items SET value = 2");
}
export async function lockAllWriteOne() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items FOR UPDATE");
  await tx.query("UPDATE items SET value = 2 WHERE id = 1");
}
export async function differentColumns() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("UPDATE items SET value = 2 WHERE parent_id = 1");
}
export async function mutableBinding(id: string) {
  const tx = await beginTransaction();
  let key = id;
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [key]);
  await tx.query("UPDATE items SET value = 2 WHERE id = $1", [id]);
}
export async function lockAllCoveredByLaterWholeRelationWrite() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items FOR UPDATE");
  await tx.query("UPDATE items SET value = 2 WHERE id = 1");
  await tx.query("UPDATE items SET value = 3");
}
