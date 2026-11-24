import { beginTransaction } from "@example/db";

export async function differentKey() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = 1 FOR UPDATE`);
  await query(`INSERT INTO jobs (id, state) VALUES (2, 'done'), (3, 'queued')
    ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`);
}

export async function differentArbiter(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(
    `INSERT INTO jobs (id, state) VALUES ($1, 'done')
    ON CONFLICT (state) DO UPDATE SET state = EXCLUDED.state`,
    [id],
  );
}

export async function doNothing(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(`INSERT INTO jobs (id, state) VALUES ($1, 'done') ON CONFLICT (id) DO NOTHING`, [id]);
}

export async function doUpdateWhere(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(
    `INSERT INTO jobs (id, state) VALUES ($1, 'done')
    ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state WHERE jobs.state = 'queued'`,
    [id],
  );
}

export async function differentTable(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE`, [id]);
  await query(
    `INSERT INTO job_audit (id, state) VALUES ($1, 'done')
    ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state`,
    [id],
  );
}
