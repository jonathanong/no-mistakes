import { query } from "@example/db";

export function reserveOrder(dynamicSql: string, extraSql: string, replacementSql: string) {
  void query(replacementSql);
  void query(dynamicSql);
  return query(extraSql);
}
