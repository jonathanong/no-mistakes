import { beginTransaction } from "@example/db";
export async function returnBeforeWrite(id: string, skip: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  if (skip) return;
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function throwBeforeWrite(id: string, skip: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  if (skip) throw new Error("skip write");
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function writeBeforeExit(id: string, skip: boolean) {
  const tx = await beginTransaction();
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
  if (skip) return;
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
}
export async function exitAfterBoth(id: string, skip: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
  if (skip) throw new Error("already written");
}
export async function nestedReturnDoesNotExitOuter(id: string) {
  const tx = await beginTransaction();
  function nested() { return; }
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function returnedWriteExecutesBeforeExit(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  return tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
