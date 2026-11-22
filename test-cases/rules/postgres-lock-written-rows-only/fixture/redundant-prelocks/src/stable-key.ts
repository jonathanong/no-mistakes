import { beginTransaction } from "@example/db";

export async function stableKeyWithMutablePayload(id: number, payload: { text: string }) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE jobs SET payload = $1 WHERE id = $2 AND state = 'running'", [payload, id]);
}

export async function mutableKey(key: { toPostgres(): number }) {
  await using query = await beginTransaction();
  await query("SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE", [key]);
  await query("UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running'", [key]);
}
