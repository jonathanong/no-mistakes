import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const rec = Object.freeze({ vote: Object.freeze({ table: "users" }) });
function inspect(value: unknown) { return new Error(String(value)); }
function getConfig(key: string) {
  if (!Object.hasOwn(rec, key)) throw inspect(rec[key as keyof typeof rec]);
  return rec[key as keyof typeof rec];
}
export function run() {
  const config = getConfig("vote");
  return query(rawSql`SELECT id FROM ${config.table}`);
}
