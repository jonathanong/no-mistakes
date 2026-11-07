import type { TransactionQuery } from "@example/db";
declare function unknownTag(parts: TemplateStringsArray, ...values: unknown[]): string;
export async function interpolatedWrite(q: TransactionQuery, id: number, otherId: number) {
  await q(`UPDATE users SET value = 1 WHERE id = ${otherId}`, [id]);
}
export async function interpolatedLock(q: TransactionQuery, id: number, otherId: number) {
  await q(`SELECT 1 FROM users WHERE id = ${otherId} FOR UPDATE`, [id]);
}
export async function taggedWrite(q: TransactionQuery, id: number) {
  await q(unknownTag`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function taggedInterpolatedWrite(q: TransactionQuery, id: number, otherId: number) {
  await q(unknownTag`UPDATE users SET value = 1 WHERE id = ${otherId}`, [id]);
}
export async function taggedLock(q: TransactionQuery, id: number) {
  await q(unknownTag`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
}
export async function stringWrite(q: TransactionQuery, id: number) {
  await q("UPDATE users SET value = 1 WHERE id = $1", [id]);
}
export async function templateWrite(q: TransactionQuery, id: number) {
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function mixedWrite(q: TransactionQuery, id: number) {
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
  await q(unknownTag`SELECT 1`);
}
export async function literalLock(q: TransactionQuery) {
  await q("SELECT 1 FROM users WHERE id = $1 FOR UPDATE", [42]);
}
