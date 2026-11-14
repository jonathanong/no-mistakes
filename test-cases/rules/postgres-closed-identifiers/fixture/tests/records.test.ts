import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

export function correlatedRecord(kind: "user" | "post") {
  const CONFIG = {
    user: { table: "users", column: "user_id" },
    post: { table: "posts", column: "post_id" },
  } as const;
  const config = CONFIG[kind];
  return query(
    rawSql`/* correlatedRecord */ DELETE FROM ${config.table} WHERE ${config.column} = $1`,
  );
}

export function repeatedChoice(table: "users" | "posts") {
  return query(
    rawSql`/* repeatedChoice */ SELECT 1 FROM ${table} AS a JOIN ${table} AS b ON a.id = b.id WHERE a.id = $1`,
  );
}
