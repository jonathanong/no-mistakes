import { query } from "@bench/db";
export async function unrelated00() {
  return query("SELECT 0");
}
