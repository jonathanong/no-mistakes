import { query } from "@bench/db";
export async function unrelated07() {
  return query("SELECT 7");
}
