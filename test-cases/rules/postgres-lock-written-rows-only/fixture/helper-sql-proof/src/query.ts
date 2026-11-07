import { beginTransaction, type TransactionQuery } from "@example/db";
declare function unknownTag(parts: TemplateStringsArray, ...values: unknown[]): string;
async function interpolatedWrite(q: TransactionQuery, id: number, otherId: number) {
  await q(`UPDATE users SET value = 1 WHERE id = ${otherId}`, [id]);
}
async function interpolatedLock(q: TransactionQuery, id: number, otherId: number) {
  await q(`SELECT 1 FROM users WHERE id = ${otherId} FOR UPDATE`, [id]);
}
async function taggedWrite(q: TransactionQuery, id: number) {
  await q(unknownTag`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function taggedInterpolatedWrite(q: TransactionQuery, id: number, otherId: number) {
  await q(unknownTag`UPDATE users SET value = 1 WHERE id = ${otherId}`, [id]);
}
async function taggedLock(q: TransactionQuery, id: number) {
  await q(unknownTag`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
}
async function stringWrite(q: TransactionQuery, id: number) {
  await q("UPDATE users SET value = 1 WHERE id = $1", [id]);
}
async function templateWrite(q: TransactionQuery, id: number) {
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function callerinterpolatedWrite(id: number, otherId: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await interpolatedWrite(tx.query, id, otherId);
}
export async function callertaggedWrite(id: number, otherId: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await taggedWrite(tx.query, id, otherId);
}
export async function callertaggedInterpolatedWrite(id: number, otherId: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await taggedInterpolatedWrite(tx.query, id, otherId);
}
export async function callerstringWrite(id: number, otherId: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await stringWrite(tx.query, id, otherId);
}
export async function callertemplateWrite(id: number, otherId: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await templateWrite(tx.query, id, otherId);
}
export async function callerinterpolatedLock(id: number, otherId: number) {
  const tx = await beginTransaction();
  await interpolatedLock(tx.query, id, otherId);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function callertaggedLock(id: number, otherId: number) {
  const tx = await beginTransaction();
  await taggedLock(tx.query, id, otherId);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function mixedWrite(q: TransactionQuery, id: number) {
  await q(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
  await q(unknownTag`SELECT 1`);
}
export async function callermixedWrite(id: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await mixedWrite(tx.query, id);
}
async function literalLock(q: TransactionQuery) {
  await q("SELECT 1 FROM users WHERE id = $1 FOR UPDATE", [42]);
}
export async function callerliteralLock() {
  const tx = await beginTransaction();
  await literalLock(tx.query);
  await tx.query("UPDATE users SET value = 1 WHERE id = $1", [42]);
}
async function interpolatedRelationLock(q: TransactionQuery, first: string, second: string, id: number) {
  await q(`SELECT 1 FROM ${first} WHERE id = $1 FOR UPDATE`, [id]);
  await q(`UPDATE ${second} SET value = 1 WHERE id = $1`, [id]);
}
export async function callerinterpolatedRelationLock(id: number) {
  const tx = await beginTransaction();
  await interpolatedRelationLock(tx.query, "users", "memberships", id);
}
