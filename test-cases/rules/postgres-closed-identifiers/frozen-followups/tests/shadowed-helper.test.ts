import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const record = Object.freeze({ vote: "users", post: "posts" });

function select(record: { vote: string; post: string }, key: string) {
  if (!Object.hasOwn(record, key)) throw new Error("no");
  return record[key as keyof typeof record];
}

export function run(kind: "vote" | "post", source: Record<string, string>) {
  return query(rawSql`SELECT id FROM ${select(source, kind)}`);
}
