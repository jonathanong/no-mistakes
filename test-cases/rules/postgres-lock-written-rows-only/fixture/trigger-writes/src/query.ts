import { beginTransaction } from "@example/db";

export async function afterInsert(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO log (id, r_id) VALUES ($2, $1)", [id, next]);
}

export async function updateOfWritten(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE updates SET selected = $2 WHERE id = $1", [id, next]);
}

export async function twoHop(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM chain_final WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO chain_start (id, r_id) VALUES ($2, $1)", [id, next]);
}

export async function cycle(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM cycle_middle WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE cycle_start SET latest = $2 WHERE id = $1", [id, next]);
}

export async function noTriggerWrites(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM untouched_r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO plain_log (id, r_id) VALUES ($2, $1)", [id, next]);
}

export async function sourceNeverWritten(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM unused_r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO plain_log (id, r_id) VALUES ($2, $1)", [id, next]);
}

export async function updateOfMismatch(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE updates SET other = $2 WHERE id = $1", [id, next]);
}

export async function wrongEvent(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("DELETE FROM log WHERE id = $1", [id, next]);
}

export async function unparseableBody(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM broken_r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO broken_log (id, r_id) VALUES ($2, $1)", [id, next]);
}

export async function conditionalTrigger(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM conditional_r WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO conditional_log (id, r_id) VALUES ($2, $1)", [id, next]);
}
