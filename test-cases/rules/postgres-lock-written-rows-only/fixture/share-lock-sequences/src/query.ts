import type { TransactionQuery } from "@example/db";
import { beginTransaction, query as poolQuery } from "@example/db";

export async function shareThenUpdate(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function keyShareThenDelete(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR KEY SHARE`, [id]);
  await query(`DELETE FROM accounts WHERE id = $1`, [id]);
}

export async function sameBatch(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE; UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function noTransaction(id: number) {
  await poolQuery(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await poolQuery(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function poolOnly(id: number) {
  await poolQuery(`SELECT 1 FROM accounts WHERE id = $1 FOR KEY SHARE`, [id]);
}

export async function writeBeforeLock(id: number) {
  await using query = await beginTransaction();
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
}

export async function differentRelation(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE account_events SET touched = true WHERE id = $1`, [id]);
}

export async function strongerLocks(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR UPDATE`, [id]);
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR NO KEY UPDATE`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function noWrite(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
}

export async function conditionalWrite(id: number, shouldWrite: boolean) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  if (shouldWrite) await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}


export async function unusedLockingCte(id: number) {
  await using query = await beginTransaction();
  await query(`WITH unused AS (SELECT 1 FROM jobs WHERE id = $1 FOR SHARE) SELECT 1`, [id]);
  await query(`UPDATE jobs SET touched = true WHERE id = $1`, [id]);
}

export async function referencedLockingCte(id: number) {
  await using query = await beginTransaction();
  await query(`WITH locked AS (SELECT 1 FROM jobs WHERE id = $1 FOR SHARE) SELECT * FROM locked`, [id]);
  await query(`UPDATE jobs SET touched = true WHERE id = $1`, [id]);
}

export async function topLevelLockWithCte(id: number) {
  await using query = await beginTransaction();
  await query(`WITH unused AS (SELECT 1) SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET touched = true WHERE id = $1`, [id]);
}


async function lockAccount(query: typeof poolQuery, id: number) {
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
}

export async function shareInHelper(id: number) {
  await using query = await beginTransaction();
  await lockAccount(query, id);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function shareThenDelete(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(`DELETE FROM accounts WHERE id = $1`, [id]);
}

export async function keyShareThenUpdate(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR KEY SHARE`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function borrowedShare(query: TransactionQuery, id: number) {
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function sameBatchWriteFirst(id: number) {
  await using query = await beginTransaction();
  await query(`UPDATE accounts SET touched = true WHERE id = $1; SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
}

export async function joinedUnknown(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts JOIN account_events ON accounts.id = account_events.account_id FOR SHARE`);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function differentExecutors(id: number) {
  await using query = await beginTransaction();
  await using other = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await other(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function suppressedUpgrade(id: number) {
  await using query = await beginTransaction();
  // no-mistakes-disable-next-line postgres-lock-written-rows-only: reviewed transaction protocol
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}
