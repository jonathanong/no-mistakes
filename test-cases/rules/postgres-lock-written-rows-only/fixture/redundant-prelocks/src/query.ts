import { beginTransaction, query as poolQuery } from "@example/db";

export async function matchingConditionalUpdate(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function matchingUpsert(id: number, state: string) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state WHERE jobs.id = $1`, [id, state]);
}

export async function selectedValueConsumed(id: number) {
  await using query = await beginTransaction();
  const rows = await query(`SELECT state FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = $2 WHERE id = $1 AND state = 'running'`, [id, rows[0].state]);
}

export async function interveningRead(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`SELECT state FROM jobs WHERE id = $1`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function differentKey(id: number, otherId: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $2 AND state = 'running'`, [id, otherId]);
}

export async function unconditionalUpdate(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1`, [id]);
}

export async function conflictDoNothing(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id) VALUES ($1) ON CONFLICT (id) DO NOTHING`, [id]);
}

export async function computedSelectedValue(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT coalesce(state, 'queued') FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function mutableKey(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  id += 1;
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function poolCalls(id: number) {
  await poolQuery(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await poolQuery(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}


export async function aliasedKeyIsDifferentColumn(key: number) {
  await using query = await beginTransaction();
  // The underlying jobs column order is id, external_id, state.
  await query(`SELECT 1 FROM ONLY jobs AS j(original_id, id, state) WHERE j.id = $1 FOR UPDATE`, [key]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [key]);
}

export async function qualifiedKeyStillMatches(key: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs AS j WHERE j.id = $1 FOR UPDATE`, [key]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [key]);
}

export async function objectKey() {
  await using query = await beginTransaction();
  const id = { value: 1 };
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function conditionalDelete(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`DELETE FROM jobs WHERE id = $1 AND state = 'running'`, [id]);
}

export async function suppressedPrelock(id: number) {
  await using query = await beginTransaction();
  // no-mistakes-disable-next-line postgres-lock-written-rows-only: reviewed transaction protocol
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function qualifyOnlyWrite(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY accounts WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE public.accounts SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function qualifyOnlyLock(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY public.accounts WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE accounts SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function differentExplicitSchemas(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY public.accounts WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE archive.accounts SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function matchingExplicitSchemas(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY "schema.with.dot".accounts WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE "schema.with.dot".accounts SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function unresolvedJoinedLockRelation(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY accounts JOIN account_events ON accounts.id = account_events.account_id WHERE accounts.id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE public.accounts SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}
