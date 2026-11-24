import { beginTransaction } from "@example/db";

export async function differentRows(id: number, other: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $2`, [id, other]);
}

export async function alreadyWriteLocked(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1`, [id]);
}

export async function weakerLockThenDelete(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR NO KEY UPDATE`, [id]);
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR KEY SHARE`, [id]);
  await query(`DELETE FROM jobs WHERE id = $1`, [id]);
}

export async function strongLockOnOtherRow(id: number, other: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR UPDATE`, [other]);
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1`, [id]);
}

export async function batchAlreadyWriteLocked(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR UPDATE; SELECT 1 FROM jobs WHERE id = $1 FOR SHARE; DELETE FROM jobs WHERE id = $1`, [id]);
}

export async function unresolvedRow(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE rank > 0`);
}

export async function skippedWriteLock(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR UPDATE SKIP LOCKED`, [id]);
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`DELETE FROM jobs WHERE id = $1`, [id]);
}
