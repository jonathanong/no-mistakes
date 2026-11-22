import { beginTransaction } from "@example/db";

export async function emptyUpsertAfterOffset(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, 'done') OFFSET 1 ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`, [id]);
}

export async function emptyUpsertAfterLimit(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, 'done') LIMIT 0 ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`, [id]);
}

export async function emptyUpsertAfterFetch(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, 'done') FETCH FIRST 0 ROWS ONLY ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`, [id]);
}
