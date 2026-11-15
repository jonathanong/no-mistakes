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

export function transformed(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${column(kind === "user" ? "post" : "user")} = $1`);
}

function flip(value: "user" | "post") {
  switch (value) {
    case "user":
      return "post";
    case "post":
      return "user";
  }
}

export function reassigned(kind: "user" | "post") {
  let k = kind;
  k = flip(k);
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${column(k)} = $1`);
}

export function unmatched() {
  return query(rawSql`DELETE FROM ${column("missing")} WHERE id = $1`);
}

function pick(kind: "user" | "post") {
  switch (kind) {
    case "user":
      return "user_id";
    case "post":
      return "post_id";
  }
}

type Key = "user" | "post";

export function castArg(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${pick(kind as Key)} = $1`);
}

export function trimmed(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${pick(kind.trim())} = $1`);
}

export function templated(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${pick(`${kind}`)} = $1`);
}

function admitNone(value: string) {
  const allowed = new Set<string>([]);
  if (!allowed.has(value)) throw new Error("closed");
  return value;
}

export function emptyGuard(kind: "user" | "post") {
  return query(rawSql`DELETE FROM ${CONFIG[kind].table} WHERE ${admitNone(kind)} = $1`);
}
