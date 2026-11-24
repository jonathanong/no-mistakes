import { beginTransaction } from "@example/db";

export async function returnedMemberWrite(id: number) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  return await tx.query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function writeAfterAwaitingValue(id: number, ready: Promise<void>) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await ready;
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function writeInMultipleDeclarators(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  const rows = await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]),
    count = rows.length;
  return count;
}

export async function computedExecutorMember(id: number) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await tx["query"](`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function separateBindArray(id: number) {
  await using query = await beginTransaction();
  const binds = [id];
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, binds);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, binds);
}

export async function separateSqlString(id: number) {
  await using query = await beginTransaction();
  const lock = "SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE";
  await query(lock, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function adjacentWriteInDifferentTransaction(id: number) {
  await using lockQuery = await beginTransaction();
  await using writeQuery = await beginTransaction();
  await lockQuery(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await writeQuery(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function writeAfterEarlierInitializer(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  const current = await query(`SELECT state FROM jobs WHERE id = $1`, [id]),
    changed = await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
  return [current, changed];
}
