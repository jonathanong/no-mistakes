import { beginTransaction } from "@example/db";

export async function laterReadMayRelyOnLock(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
  return await query(`SELECT state FROM jobs WHERE id = $1`, [id]);
}

export async function laterQualifiedWriteMayRelyOnLock(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
  await query(`DELETE FROM public.jobs WHERE id = $1`, [id]);
}

export async function laterReadOfAnotherRelation(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
  return await query(`SELECT state FROM job_events WHERE job_id = $1`, [id]);
}

export async function laterReadInAnotherTransaction(id: number) {
  await using query = await beginTransaction();
  await using other = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
  return await other(`SELECT state FROM jobs WHERE id = $1`, [id]);
}
