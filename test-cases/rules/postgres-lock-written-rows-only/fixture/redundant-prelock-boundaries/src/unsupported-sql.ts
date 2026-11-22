import { beginTransaction } from "@example/db";

export async function parenthesizedLock(id: number) {
  await using query = await beginTransaction();
  await query(`(SELECT 1 FROM ONLY jobs WHERE id = $1) FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function multipleProjections(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1, 2 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function multipleLockClauses(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR SHARE OF jobs FOR UPDATE OF jobs`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function multipleRelations(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs, job_events WHERE jobs.id = $1 FOR UPDATE OF jobs`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function updateFromAnotherRelation(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' FROM job_events WHERE jobs.id = $1 AND jobs.id = job_events.job_id`, [id]);
}

export async function rangePredicate(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id >= $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function computedAssignment(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = upper(state) WHERE id = $1 AND state = 'running'`, [id]);
}

export async function derivedLockTarget(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM (SELECT id FROM ONLY jobs) AS j WHERE j.id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function explicitInheritedRows(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs * WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function multiColumnConflictTarget(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, 'done') ON CONFLICT (id, state) DO UPDATE SET state = EXCLUDED.state`, [id]);
}

export async function insertSelect(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) SELECT $1, 'done' ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`, [id]);
}

export async function multipleInsertedRows(id: number, otherId: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, 'done'), ($2, 'done') ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`, [id, otherId]);
}

export async function unsupportedLockOnUnrelatedTable(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1, 2 FROM job_events WHERE job_id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}
