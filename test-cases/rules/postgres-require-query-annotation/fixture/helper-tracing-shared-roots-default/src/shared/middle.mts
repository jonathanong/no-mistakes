import { formatSql } from "./leaf.mjs";

export function buildSelect(table: string) {
  return formatSql("SELECT id FROM " + table);
}
