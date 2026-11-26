import { beginTransaction } from "@example/db";

export async function createChild(id: number, note: string) {
  await using query = await beginTransaction();
  await query("CREATE TABLE task_copy () INHERITS (tasks)");
  await query("SELECT id FROM tasks WHERE id = $1 FOR UPDATE", [id]);
  await query("INSERT INTO tasks (id, note) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note", [id, note]);
}

export async function attachChild(id: number, note: string) {
  await using query = await beginTransaction();
  await query("ALTER TABLE task_copy INHERIT tasks");
  await query("SELECT id FROM tasks WHERE id = $1 FOR UPDATE", [id]);
  await query("INSERT INTO tasks (id, note) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note", [id, note]);
}
