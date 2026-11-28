import { query } from "@example/db";

export async function lockOrder(id: number) {
  await query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
}
