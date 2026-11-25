import { beginTransaction } from "@example/db";

export async function siblingComplexUpdate(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`UPDATE jobs SET state = 'pending' WHERE state = 'queued' AND touched_at IS NULL`);
  await query(
    `INSERT INTO jobs (id, state) VALUES ($1, 'done')
    ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`,
    [id],
  );
}
