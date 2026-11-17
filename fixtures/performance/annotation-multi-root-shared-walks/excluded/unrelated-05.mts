import { query } from "@bench/db";
export async function unrelated05() {
  return query("SELECT 5");
}
