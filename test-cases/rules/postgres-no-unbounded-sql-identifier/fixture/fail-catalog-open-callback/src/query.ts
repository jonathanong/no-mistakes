import { query } from "@example/db";
import { TABLES } from "./catalog";

export function suffixed(suffix: string) {
  const refs = TABLES.map((table) => `${table}_${suffix}`).join(", ");
  return query(`SELECT 1 FROM ${refs}`);
}

export function joinedParameter(names: string[]) {
  const list = names.join(", ");
  return query(`SELECT 1 FROM ${list}`);
}

export function openAlias(table: string) {
  for (const known of TABLES) {
    void known;
  }
  const t = table;
  return query(`SELECT 1 FROM ${t}`);
}
