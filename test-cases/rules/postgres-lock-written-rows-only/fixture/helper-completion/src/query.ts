import { beginTransaction, type TransactionQuery } from "@example/db";
async function writeCallable(q: TransactionQuery, id: string) {
  await q(`SELECT 1`);
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function writeObject(q: TransactionQuery, id: string) {
  await q.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function writeBare(q: TransactionQuery, id: string) {
  q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function writeMixed(q: TransactionQuery, id: string) {
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
  await q.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function callerCallableValid(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await writeCallable(tx.query, id);
}
export async function callerObjectValid(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await writeObject(tx, id);
}
export async function callerReturnAwaitValid(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  return await writeCallable(tx.query, id);
}
export async function callerCallableShape(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await writeCallable(tx, id);
}
export async function callerObjectShape(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await writeObject(tx.query, id);
}
export async function callerMixedShape(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await writeMixed(tx.query, id);
}
export async function callerUnawaited(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  void writeCallable(tx.query, id);
}
export async function callerReturned(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  return writeCallable(tx.query, id);
}
export async function callerBareExecutor(id: string) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await writeBare(tx.query, id);
}
