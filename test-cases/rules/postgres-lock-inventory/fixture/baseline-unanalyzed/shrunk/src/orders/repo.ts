import { query } from "@example/db";

export function reserveOrder(dynamicSql: string) {
  return query(dynamicSql);
}
