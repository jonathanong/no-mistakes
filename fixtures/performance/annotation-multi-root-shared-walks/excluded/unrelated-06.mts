import { query } from "@bench/db";
export async function unrelated06() {
  return query("SELECT 6");
}
