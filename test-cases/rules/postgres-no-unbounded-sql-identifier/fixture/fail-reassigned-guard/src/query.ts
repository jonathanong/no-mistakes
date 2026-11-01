import { query } from "@example/db";

declare function assertWhitelistedSqlIdentifier(value: string): void;

const SET = new Set(["users", "posts"]);

export function assignAfterGuard(u: string) {
  if (!SET.has(u)) throw new Error("table");
  u = process.env.T;
  return query(`SELECT id FROM ${u}`);
}

export function assertThenAssign(u: string) {
  assertWhitelistedSqlIdentifier(u);
  u = process.env.T;
  return query(`SELECT id FROM ${u}`);
}

export function assertThenUpdate(u: string) {
  assertWhitelistedSqlIdentifier(u);
  u += process.env.T;
  return query(`SELECT id FROM ${u}`);
}

export function forOfAfterGuard(u: string) {
  if (!SET.has(u)) throw new Error("table");
  for (u of [process.env.T]) {
    return query(`SELECT id FROM ${u}`);
  }
}

export function destructureAfterGuard(u: string) {
  if (!SET.has(u)) throw new Error("table");
  ({ u } = { u: process.env.T });
  return query(`SELECT id FROM ${u}`);
}

export function innerShadow(u: string) {
  if (!SET.has(u)) throw new Error("table");
  {
    const u = process.env.T;
    return query(`SELECT id FROM ${u}`);
  }
}

export function writeBeforeGuard(u: string) {
  u = "users";
  if (!SET.has(u)) throw new Error("table");
  return query(`SELECT id FROM ${u}`);
}
