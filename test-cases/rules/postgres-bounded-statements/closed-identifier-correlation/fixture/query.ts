import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const CONFIG = {
  user: { table: "users", column: "user_id" },
  post: { table: "posts", column: "post_id" },
} as const;

function column(kind: "user" | "post") {
  switch (kind) {
    case "user":
      return "user_id";
    case "post":
      return "post_id";
  }
}

function table(kind: "user" | "post") {
  switch (kind) {
    case "user":
      return "users";
    case "post":
      return "posts";
  }
}

export function correlated(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${column(kind)} = $1`);
}

export function twoHelpers(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${table(kind)} WHERE ${column(kind)} = $1`);
}

export function uncorrelated(kind: "user" | "post", other: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${column(other)} = $1`);
}
