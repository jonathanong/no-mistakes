import { query } from "@bench/db";
export async function unrelated02() {
  return query("SELECT 2");
}
