import { query } from "@example/db";

const TABLES = new Map([
  ["user", "users"],
  ["post", "posts"],
]);

const FROZEN_TABLES = Object.freeze(
  new Map([
    ["user", "users"],
    ["post", "posts"],
  ]),
);

export function lookup(kind: string) {
  const table = TABLES.get(kind);
  return query(`SELECT id FROM ${table}`);
}

export function inline(kind: string) {
  return query(`SELECT id FROM ${TABLES.get(kind)}`);
}

export function literalGet() {
  return query(`SELECT id FROM ${TABLES.get("user")}`);
}

export function frozenLookup(kind: string) {
  const table = FROZEN_TABLES.get(kind);
  return query(`SELECT id FROM ${table}`);
}
