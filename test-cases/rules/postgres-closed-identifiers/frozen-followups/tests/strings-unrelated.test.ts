import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const names = Object.freeze({ vote: "users", post: "posts" });

function guarded(key: string) {
  if (!Object.hasOwn(names, key)) throw new Error("unknown table");
  return names[key as keyof typeof names];
}

function unrelated(names) {
  return names;
}

export function run() {
  {
    let names = { vote: "audit" };
    void names;
  }
  const copy = unrelated({ vote: "ignored", post: "ignored" });
  void copy;
  return query(rawSql`SELECT id FROM ${guarded("vote")} WHERE id = $1`);
}
