import { query } from "@example/db";
export async function claim() {
  await query(`WITH candidates AS (SELECT id FROM jobs ORDER BY id LIMIT 50 FOR UPDATE SKIP LOCKED), claimed AS (UPDATE jobs SET claimed_at = now() FROM candidates WHERE jobs.id = candidates.id RETURNING jobs.id) SELECT id FROM claimed`);
  await query(`WITH candidates AS (SELECT id FROM jobs ORDER BY id LIMIT 50 FOR UPDATE SKIP LOCKED) UPDATE jobs SET claimed_at = now() FROM candidates WHERE jobs.id = candidates.id`);
  await query(`WITH candidates AS (SELECT id FROM jobs FOR UPDATE), removed AS (DELETE FROM jobs USING candidates WHERE jobs.id = candidates.id RETURNING jobs.id) SELECT id FROM removed`);
  await query(`WITH candidates AS (SELECT id FROM jobs FOR UPDATE), changed AS (UPDATE logs SET value = 1 RETURNING id) SELECT id FROM candidates`);
  await query(`SELECT id FROM jobs FOR UPDATE; UPDATE jobs SET claimed_at = now()`);
  await query(`SELECT id FROM jobs FOR UPDATE`);
  await query(`WITH jobs AS (SELECT id FROM logs), candidates AS (SELECT id FROM jobs FOR UPDATE), changed AS (UPDATE jobs SET claimed_at = now() RETURNING id) SELECT id FROM candidates`);
}
