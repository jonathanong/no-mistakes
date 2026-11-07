import { query } from "@example/db";
export async function autoCommit(id: string) {
  await query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await query(`SELECT id FROM items WHERE id = $1 FOR SHARE`, [id]);
}
