import { query } from "@example/db";

export function reserveOrder(id: string, dynamicSql: string, extraSql: string) {
  void query("SELECT id FROM orders WHERE id = $1 FOR UPDATE", [id]);
  void query(dynamicSql);
  return query(extraSql);
}
