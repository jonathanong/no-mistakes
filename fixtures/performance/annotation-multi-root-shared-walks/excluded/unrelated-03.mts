import { query } from "@bench/db";
export async function unrelated03() {
  return query("SELECT 3");
}
