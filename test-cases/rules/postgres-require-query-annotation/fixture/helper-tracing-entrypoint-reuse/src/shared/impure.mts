import { query } from "@bench/db";

export const bootResult = query("SELECT id FROM boot_items");

export function runImpure(sql: string) {
  return query(sql);
}
