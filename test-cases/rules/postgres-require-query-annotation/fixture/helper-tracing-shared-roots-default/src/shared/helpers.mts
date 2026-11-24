import { buildSelect } from "./middle.mjs";
import { leafPrefix } from "./leaf.mjs";

export function runAnnotated(table: string, exec: (sql: string) => unknown) {
  return exec(buildSelect(table));
}

export function runRaw(sql: string, exec: (sql: string) => unknown) {
  return exec(sql);
}

export const helperPrefix = leafPrefix;
