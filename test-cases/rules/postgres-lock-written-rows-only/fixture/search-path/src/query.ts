import { beginTransaction } from "@example/db";
export async function changeSearchPath() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("SET LOCAL search_path = other_schema");
  await tx.query("UPDATE items SET value = 2 WHERE id = 1");
}
export async function changeSearchPathWithFunction() {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("SELECT set_config('search_path', 'other_schema', true)");
  await tx.query("UPDATE items SET value = 2 WHERE id = 1");
}
