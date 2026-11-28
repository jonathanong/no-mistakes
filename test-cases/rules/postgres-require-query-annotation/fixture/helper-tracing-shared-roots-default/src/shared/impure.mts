import { query } from "@bench/db";

export const bootCheck = query("SELECT id FROM boot_check");

export function runImpure(sql: string) {
  return query(sql);
}
