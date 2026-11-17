import { query } from "@bench/db";
export async function unrelated04() {
  return query("SELECT 4");
}
