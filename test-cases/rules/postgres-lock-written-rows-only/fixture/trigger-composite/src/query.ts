import { beginTransaction } from "@example/db";

export async function updateTotal(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE orders SET total.amount = 1 WHERE id = $1", [id]);
}

export async function updateAmount(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM wrong_accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE orders SET total.amount = 1 WHERE id = $1", [id]);
}

export async function upsertTotal(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET total.amount = 1", [id]);
}

export async function upsertAmount(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM wrong_accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET total.amount = 1", [id]);
}

export async function chainTotal(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO chain_start (id) VALUES ($1)", [id]);
}

export async function chainAmount(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM wrong_accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO chain_start (id) VALUES ($1)", [id]);
}
