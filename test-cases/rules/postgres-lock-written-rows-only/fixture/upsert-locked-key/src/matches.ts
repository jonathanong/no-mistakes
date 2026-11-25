import { beginTransaction } from "@example/db";

export async function singleRow(id: number, touchedAt: string) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`
    INSERT INTO jobs (id, state, touched_at) VALUES ($1, 'done', $2)
    ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state, touched_at = EXCLUDED.touched_at
  `, [id, touchedAt]);
}

export async function twoRows(id: number, otherId: number, touchedAt: string) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`
    INSERT INTO jobs (id, state, touched_at)
    VALUES ($1, 'done', $3), ($2, 'queued', $3)
    ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state, touched_at = EXCLUDED.touched_at
  `, [otherId, id, touchedAt]);
}
