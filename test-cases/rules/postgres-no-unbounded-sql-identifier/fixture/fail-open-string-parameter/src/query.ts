import { query } from "@example/db";

export function open(table: string) {
  return query(`SELECT id FROM ${table}`);
}

export function suffixed(table: string, yyyy: number) {
  return query(`SELECT id FROM ${table}__p_${yyyy}`);
}

export function aliased(table: string) {
  const t = table;
  return query(`SELECT id FROM ${t}`);
}
