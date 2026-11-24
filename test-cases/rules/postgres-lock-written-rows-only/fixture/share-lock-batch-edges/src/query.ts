import { beginTransaction } from "@example/db";

export async function conditionalKeyShareThenDelete(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  if (enabled) {
    await query(`SELECT 1 FROM jobs WHERE id = $1 FOR KEY SHARE;
      SELECT 1 FROM job_events WHERE job_id = $1 FOR UPDATE;
      DELETE FROM jobs WHERE id = $1`, [id]);
  }
}

export async function conditionalDeleteThenKeyShare(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  if (enabled) {
    await query(`DELETE FROM jobs WHERE id = $1;
      SELECT 1 FROM jobs WHERE id = $1 FOR KEY SHARE`, [id]);
  }
}

export async function conditionalKeyShareThenUpdate(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  if (enabled) {
    await query(`SELECT 1 FROM jobs WHERE id = $1 FOR KEY SHARE;
      UPDATE jobs SET state = 'done' WHERE id = $1`, [id]);
  }
}

export async function repeatedShareOnSameRelation(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs FOR SHARE;
    SELECT 1 FROM jobs FOR SHARE;
    UPDATE jobs SET state = 'done' WHERE id = $1`, [id]);
}

export async function ambiguousKeyShareThenConditionalDelete(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs JOIN job_events ON jobs.id = job_events.job_id FOR KEY SHARE`);
  if (enabled) {
    await query(`DELETE FROM jobs WHERE id = $1`, [id]);
  }
}
