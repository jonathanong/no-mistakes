import { beginTransaction, type TransactionQuery } from "@example/db";
export async function lock(query: TransactionQuery, key: string) {
  await query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
}
export async function write(query: TransactionQuery, key: string) {
  await query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function callerLock(query: TransactionQuery, key: string) {
  await lock(query, key);
  await query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function callerWrite(query: TransactionQuery, key: string) {
  await query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  await write(query, key);
}
export async function callerMismatch(query: TransactionQuery, key: string) {
  await lock(query, key);
  await query(`DELETE FROM claims WHERE key = $1`, ['other']);
}
export async function valid(key: string) {
  const tx = await beginTransaction();
  await callerLock(tx.query, key);
  await callerWrite(tx.query, key);
}
export async function invalid(key: string) {
  const tx = await beginTransaction();
  await callerMismatch(tx.query, key);
}
export async function typeOnly(query: TransactionQuery, key: string) {
  await query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
}
export async function cycleOne(query: TransactionQuery, key: string) {
  await cycleTwo(query, key);
}
export async function cycleTwo(query: TransactionQuery, key: string) {
  await cycleOne(query, key);
}
export async function cyclic(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  await cycleOne(tx.query, key);
}
export async function splitTransactions(key: string) {
  const first = await beginTransaction();
  const second = await beginTransaction();
  await lock(first.query, key);
  await write(second.query, key);
}
