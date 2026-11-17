import { beginTransaction } from "@example/db";

export async function dynamicName(id: string, setting: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT set_config($1, 'replica', true)", [setting]);
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1)", [id]);
}

export async function directWrite(id: string, setting: string, value: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT pg_catalog.set_config($1, 'replica', true)", [setting]);
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE accounts SET latest = $2 WHERE id = $1", [id, value]);
}

export async function knownOtherSetting(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT set_config('application_name', 'worker', true)");
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1)", [id]);
}

export async function expressionName(id: string, setting: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT pg_catalog.set_config(lower($1), 'replica', true)", [setting]);
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1)", [id]);
}
