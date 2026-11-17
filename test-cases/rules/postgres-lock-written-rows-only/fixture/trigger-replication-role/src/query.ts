import { beginTransaction } from "@example/db";

export async function replicaSet(id: string) {
  const tx = await beginTransaction();
  await tx.query("SET LOCAL session_replication_role = replica");
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1)", [id]);
}

export async function replicaSetConfig(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT pg_catalog.set_config('session_replication_role', 'replica', true)");
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1)", [id]);
}

export async function directWrite(id: string, value: string) {
  const tx = await beginTransaction();
  await tx.query("SET LOCAL session_replication_role = replica");
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE accounts SET latest = $2 WHERE id = $1", [id, value]);
}
