import { beginTransaction } from "@example/db";
import { interpolatedWrite, interpolatedLock, taggedWrite, taggedInterpolatedWrite, taggedLock, stringWrite, templateWrite, mixedWrite, literalLock } from "./helper";
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
export async function callermixedWrite(id: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await mixedWrite(tx.query, id);
}
export async function callerliteralLock() {
  const tx = await beginTransaction();
  await literalLock(tx.query);
  await tx.query("UPDATE users SET value = 1 WHERE id = $1", [42]);
}
