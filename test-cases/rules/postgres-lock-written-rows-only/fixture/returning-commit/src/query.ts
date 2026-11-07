import { beginTransaction } from "@example/db";
async function inspect(tx: unknown) {
  await tx.query(`SELECT 1`);
}
export async function valid(key: string) {
  const tx = await beginTransaction();
  const owned = await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (owned.rowCount !== 1) {
    await tx.commit();
    return;
  }
  await inspect(tx);
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function invalid(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (key) {
    await tx.commit();
  }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function invalidUncommittedReturn(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (key) { return; }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedSqlBoundary(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (key) { await tx.query(`COMMIT`); return; }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedOtherLiveTransaction(key: string) {
  const tx = await beginTransaction();
  const other = await beginTransaction();
  const owned = await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (owned.rowCount !== 1) { await tx.commit(); return; }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
  await other.commit();
}
export async function unresolvedNestedReturn(key: string, flag: boolean, other: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (flag) {
    if (other) { return; }
    await tx.commit();
    return;
  }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedDeadCommit(key: string, flag: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (flag) {
    throw new Error('stop');
    await tx.commit();
    return;
  }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedUnawaitedCommit(key: string, flag: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (flag) { tx.commit(); return; }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedUnawaitedRollback(key: string, flag: boolean) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (flag) { tx.rollback(); return; }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedContinuingCommit(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  tx.commit();
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
export async function unresolvedContinuingRollback(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  tx.rollback();
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}

export async function validAfterIncompleteSibling(key: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM claims WHERE key = $1 FOR UPDATE`, [key]);
  if (key) { await tx.commit(); return; }
  await tx.query(`DELETE FROM claims WHERE key = $1`, [key]);
}
