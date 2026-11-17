import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const records = Object.freeze({
  user: Object.freeze({ table: "users", column: "id" }),
  post: Object.freeze({ table: "posts", column: "id" }),
});

function getConfig(key: string) {
  if (!Object.hasOwn(records, key)) throw new Error("unknown record");
  return records[key as keyof typeof records];
}

function selectKey(_group: string, key: string) {
  const allowed = new Set(["user", "post"]);
  if (!allowed.has(key)) throw new Error("unknown record");
  return key;
}

export function atBudget(group: "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h", key: string) {
  const config = getConfig(selectKey(group, key));
  return query(rawSql`SELECT '${group}' AS cohort, id FROM ${config.table} WHERE ${config.column} = $1`);
}

export function overBudget(group: "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i", key: string) {
  const config = getConfig(selectKey(group, key));
  return query(rawSql`SELECT '${group}' AS cohort, id FROM ${config.table} WHERE ${config.column} = $1`);
}
