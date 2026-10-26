import { query } from "@example/db";

export function findById(id: string) {
  return query("SELECT 1 FROM active_accounts WHERE id = $1", [id]);
}
