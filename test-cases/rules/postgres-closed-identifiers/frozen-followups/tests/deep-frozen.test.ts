import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const rec = Object.freeze({
  vote: Object.freeze({ table: "users", column: "id" }),
  post: Object.freeze({ table: "posts", column: "id" }),
});

function getConfig(key: string) {
  if (!Object.hasOwn(rec, key)) throw new Error("no");
  return rec[key as keyof typeof rec];
}

export function run() {
  const selected = getConfig("vote");
  (selected as { table: string }).table = "audit";
  const config = getConfig("vote");
  return query(rawSql`SELECT id FROM ${config.table} WHERE ${config.column} = $1`);
}
