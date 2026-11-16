import { query } from "@bench/db";
export async function unrelated01() {
  return query("SELECT 1");
}
