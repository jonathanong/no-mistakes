import { query } from "@example/db";

export async function raceTestHelper(id: number) {
  await query("SELECT id FROM refunds WHERE id = $1 FOR UPDATE", [id]);
}
