import { beginTransaction } from "@example/db";
export async function keyedLock(id: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT pg_advisory_xact_lock($1)`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
