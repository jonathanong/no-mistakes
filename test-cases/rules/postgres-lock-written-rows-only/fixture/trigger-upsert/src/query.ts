import { beginTransaction } from "@example/db";

export async function selectedUpdate(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id, selected) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET selected = EXCLUDED.selected", [id, next]);
}

export async function doNothing(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id, selected) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING", [id, next]);
}

export async function otherUpdate(id: string, next: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("INSERT INTO orders (id, selected) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET other = EXCLUDED.selected", [id, next]);
}
