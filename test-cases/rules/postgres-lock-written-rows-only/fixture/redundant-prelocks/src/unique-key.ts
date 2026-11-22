import { beginTransaction } from "@example/db";

export async function broadStateLock(id: number) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY jobs WHERE state = 'running' FOR UPDATE");
  await query("UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'", [id]);
}

export async function partialIndex(id: number) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY partial_jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE partial_jobs SET state = 'done' WHERE id = $1 AND state = 'running'", [id]);
}

export async function deferredIndex(id: number) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY deferred_jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE deferred_jobs SET state = 'done' WHERE id = $1 AND state = 'running'", [id]);
}

export async function staleIndex(id: number) {
  await using query = await beginTransaction();
  await query("ALTER TABLE stale_jobs DROP CONSTRAINT stale_jobs_id_key");
  await query("SELECT 1 FROM ONLY stale_jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE stale_jobs SET state = 'done' WHERE id = $1 AND state = 'running'", [id]);
}
