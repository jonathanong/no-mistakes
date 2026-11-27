import { query } from "@example/db";

export function reserveOrder(id: string, dynamicSql: string) {
  void query("SELECT id FROM orders WHERE id = $1 FOR UPDATE", [id]);
  return query(dynamicSql);
}
