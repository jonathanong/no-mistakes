import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const RECORDS = {
  user: { table: "users" },
  post: { table: "posts" },
} as const;

function pair(left: "user" | "post", right: "user" | "post") {
  if (left === "user" && right === "user") {
    return "users";
  }
  if (left === "post" && right === "post") {
    return "posts";
  }
  return "cross";
}

function join(a: string, b: string) {
  return a + "_" + b;
}

function fromRecord(_record: { user: { table: string }; post: { table: string } }) {
  return "literal";
}

export function diagonal(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${pair(kind, kind)} WHERE id = $1`);
}

export function underBudget(a: "a" | "b", b: "w" | "x") {
  return query(rawSql`DELETE FROM ${join(a, b)} WHERE id = $1`);
}

export function overBudget(a: "a" | "b" | "c" | "d" | "e", b: "w" | "x" | "y" | "z") {
  return query(rawSql`DELETE FROM ${join(a, b)} WHERE id = $1`);
}

export function recordArgument() {
  return query(rawSql`DELETE FROM ${fromRecord(RECORDS)} WHERE id = $1`);
}
