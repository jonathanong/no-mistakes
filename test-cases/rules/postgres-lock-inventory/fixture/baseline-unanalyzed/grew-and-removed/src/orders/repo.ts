import { query } from "@example/db";

export function reserveOrder(dynamicSql: string, replacementSql: string) {
  void query(replacementSql);
  return query(dynamicSql);
}
