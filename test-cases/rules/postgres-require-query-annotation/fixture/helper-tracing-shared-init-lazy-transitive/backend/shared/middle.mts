import { formatLeafSql } from "./leaf.mjs";

export function buildMiddleSql(table: string) {
  return formatLeafSql(table);
}
