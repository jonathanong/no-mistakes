import type { TransactionQuery } from "@example/db";
export async function writeCallable(q: TransactionQuery, id: string) {
  await q(`SELECT 1`);
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function writeObject(q: TransactionQuery, id: string) {
  await q.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function writeBare(q: TransactionQuery, id: string) {
  q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function writeMixed(q: TransactionQuery, id: string) {
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
  await q.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
