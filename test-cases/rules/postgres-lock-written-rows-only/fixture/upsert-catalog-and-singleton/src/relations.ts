import { beginTransaction } from "@example/db";

export async function plainTask(id: number, note: string) {
  await using query = await beginTransaction();
  await query("SELECT id FROM tasks WHERE id = $1 FOR UPDATE", [id]);
  await query("INSERT INTO tasks (id, note) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note", [id, note]);
}

export async function parentTask(id: number, note: string) {
  await using query = await beginTransaction();
  await query("SELECT id FROM task_parent WHERE id = $1 FOR UPDATE", [id]);
  await query("INSERT INTO task_parent (id, note) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note", [id, note]);
}

export async function unknownTask(id: number, note: string) {
  await using query = await beginTransaction();
  await query("SELECT id FROM legacy_tasks WHERE id = $1 FOR UPDATE", [id]);
  await query("INSERT INTO legacy_tasks (id, note) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note", [id, note]);
}
