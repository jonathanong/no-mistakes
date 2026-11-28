import { query } from "@example/db";

export async function suppressedCalls(id: number) {
  // no-mistakes-disable-next-line postgres-lock-reason: legacy path
  await query("SELECT id FROM jobs WHERE id = $1 FOR UPDATE", [id]);
  await query("SELECT pg_advisory_lock($1)", [id]); // no-mistakes-disable-line postgres-lock-reason: legacy advisory lock
}
