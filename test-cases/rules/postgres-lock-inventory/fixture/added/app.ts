import { query } from "@example/db";
export async function reserve(id: number) {
  await lockAccountRow(id);
  return query("SELECT id FROM accounts FOR SHARE");
}
