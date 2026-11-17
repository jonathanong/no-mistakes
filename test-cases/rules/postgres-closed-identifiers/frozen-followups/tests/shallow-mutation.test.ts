import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const rec = Object.freeze({ vote: { table: "users", column: "id" } });

function getConfig(key: string) {
  if (!Object.hasOwn(rec, key)) throw new Error("no");
  return rec[key as keyof typeof rec];
}

const e = rec["vote"];
e.table = "audit";

export function run() {
  return query(rawSql`SELECT id FROM ${getConfig("vote").table} WHERE ${getConfig("vote").column} = $1`);
}
