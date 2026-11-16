import * as pool from "@example/db";

export function claimJobs(cutoff: Date, lastId: string, limit: number) {
  const parameters = [cutoff, lastId, limit];
  const text = `SELECT id FROM jobs WHERE (created_at, id) > ($${parameters.length - 2}::timestamptz, $${parameters.length - 1}::uuid) ORDER BY created_at, id LIMIT $${parameters.length} FOR UPDATE OF jobs SKIP LOCKED`;
  return pool.query(text, parameters);
}

export function byId(n: number) {
  const text = `SELECT id FROM jobs WHERE id = $${n} FOR UPDATE`;
  return pool.query(text, [n]);
}
