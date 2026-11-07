import { beginTransaction } from "@example/db";
export async function globalLocks(id: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT pg_advisory_xact_lock(42)`);
  await tx.query(`SELECT pg_advisory_lock(1, 2)`);
  await tx.query(`SELECT pg_try_advisory_xact_lock_shared(7)`);
  await tx.query(`SELECT pg_try_advisory_lock(3, 4)`);
  await tx.query(`SELECT pg_advisory_xact_lock($1)`, [42]);
  const globalKey = 99;
  await tx.query(`SELECT pg_advisory_xact_lock($1)`, [globalKey]);
}
