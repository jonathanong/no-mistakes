import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

function Object() {
  return null;
}

const tables = Object.freeze({
  vote: "users",
  tag: "posts",
} as const);

function getTable(name: string) {
  if (!Object.hasOwn(tables, name)) throw new Error("no");
  return tables[name as keyof typeof tables];
}

export function run(kind: string) {
  return query(rawSql`/* shadow */ DELETE FROM ${getTable(kind)} WHERE id = $1`);
}
