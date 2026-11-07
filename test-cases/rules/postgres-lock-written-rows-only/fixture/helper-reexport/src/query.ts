import { beginTransaction } from "@example/db";
import { acquireUser, lockConstant, lockUnknown } from "./barrel";
export async function written(id: string) {
  const tx = await beginTransaction();
  await acquireUser(tx.query, id);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
export async function mismatch(id: string, otherId: string) {
  const tx = await beginTransaction();
  await acquireUser(tx.query, id);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [otherId]);
}
export async function constantWritten() {
  const tx = await beginTransaction();
  await lockConstant(tx.query);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [42]);
}
export async function unknown(id: string) {
  const tx = await beginTransaction();
  await lockUnknown(tx.query, id);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
