import { beginTransaction } from "@example/db";

export async function laterRead(id: number, state: string) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY "JobQueue" WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO "JobQueue" AS job (id, state) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state WHERE job.id = $1`, [id, state]);
  return await query(`SELECT state FROM "JobQueue" WHERE id = $1`, [id]);
}

export async function noLaterRead(id: number, state: string) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY "JobQueue" WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO "JobQueue" AS job (id, state) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state WHERE job.id = $1`, [id, state]);
}
