import { query as pool } from "@example/db";
export async function rejected(id: string) {
  const q = pool;
  await q(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
}
