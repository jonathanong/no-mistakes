import { beginTransaction } from "@example/db";

export async function inheritedReadBeforeUpdate(id: number) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'", [id]);
}

export async function inheritedReadBeforeUpsert(id: number) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("INSERT INTO jobs (id, state) VALUES ($1, 'done') ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state", [id]);
}
