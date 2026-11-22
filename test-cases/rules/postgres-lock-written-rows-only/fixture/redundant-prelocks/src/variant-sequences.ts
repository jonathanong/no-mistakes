import { beginTransaction } from "@example/db";

export async function variantWrite(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(enabled ? `UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'` : `SELECT 1 FROM jobs WHERE id = $1`, [id]);
}

export async function variantLock(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(enabled ? `SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE` : `SELECT 1 FROM jobs WHERE id = $1`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'`, [id]);
}

export async function unrelatedVariantWrite(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(enabled ? `UPDATE job_events SET state = 'done' WHERE id = $1 AND state = 'running'` : `SELECT 1 FROM jobs WHERE id = $1`, [id]);
}
