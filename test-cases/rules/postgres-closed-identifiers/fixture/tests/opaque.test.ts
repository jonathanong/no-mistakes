import { query, sql } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";
import {
  assertWhitelistedSqlIdentifier as allowed,
  assertSafeSqlIdentifier,
} from "@example/identifiers";
import { assertWhitelistedSqlIdentifier as other } from "@other/identifiers";

declare function unknownTable(): string;
declare function escape(value: unknown): void;

function mixed(kind: "user" | "post") {
  return kind === "user" ? "users" : unknownTable();
}

export function mixedBranch(kind: "user" | "post") {
  return query(sql`/* mixedBranch */ SELECT 1 FROM `.append(mixed(kind)));
}

function incomplete(kind: "user" | "post") {
  switch (kind) {
    case "user":
      return "users";
  }
}

export function missingBranch(kind: "user" | "post") {
  return query(sql`/* missingBranch */ SELECT 1 FROM `.append(incomplete(kind)));
}

export function mixedAllowlist(table: string) {
  return query(
    sql`/* mixedAllowlist */ SELECT 1 FROM `.append(allowed(table, ["users", unknownTable()])),
  );
}

export function noAllowlist(table: string) {
  return query(rawSql`/* noAllowlist */ LOCK TABLE ${assertSafeSqlIdentifier(table)}`);
}

export function wrongModule(table: string) {
  return query(sql`/* wrongModule */ SELECT 1 FROM `.append(other(table, ["users", "posts"])));
}

export function shadowedHelper(
  allowed: (table: string, values: string[]) => string,
  table: string,
) {
  return query(sql`/* shadowedHelper */ SELECT 1 FROM `.append(allowed(table, ["users", "posts"])));
}

export function reassigned(table: "users" | "posts") {
  table = unknownTable() as any;
  return query(sql`/* reassigned */ SELECT 1 FROM `.append(table));
}

export function castOnly(table: string) {
  const narrowed = table as "users" | "posts";
  return query(sql`/* castOnly */ SELECT 1 FROM `.append(narrowed));
}

export function escapedSet(table: string) {
  const names = new Set(["users", "posts"]);
  escape(names);
  return query(sql`/* escapedSet */ SELECT 1 FROM `.append(allowed(table, names)));
}

export function mutatedSet(table: string) {
  const names = new Set(["users", "posts"]);
  names.add(unknownTable());
  return query(sql`/* mutatedSet */ SELECT 1 FROM `.append(allowed(table, names)));
}

export function changedRecord(kind: "user" | "post") {
  const CONFIG = { user: { table: "users" }, post: { table: "posts" } } as const;
  const config = CONFIG[kind];
  (config as any).table = unknownTable();
  return query(rawSql`/* changedRecord */ SELECT 1 FROM ${config.table}`);
}

export function missingRecordKey(kind: "user" | "post" | "missing") {
  const CONFIG = { user: { table: "users" }, post: { table: "posts" } } as const;
  return query(rawSql`/* missingRecordKey */ SELECT 1 FROM ${CONFIG[kind].table}`);
}

export function oversized(
  table:
    | "t01"
    | "t02"
    | "t03"
    | "t04"
    | "t05"
    | "t06"
    | "t07"
    | "t08"
    | "t09"
    | "t10"
    | "t11"
    | "t12"
    | "t13"
    | "t14"
    | "t15"
    | "t16"
    | "t17",
) {
  return query(sql`/* oversized */ SELECT 1 FROM `.append(table));
}

export function escapedChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  escape(config.child);
  return query(rawSql`/* escapedChild */ SELECT 1 FROM ${config.child.table}`);
}

export function assignedChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  Object.assign(config.child, { table: unknownTable() });
  return query(rawSql`/* assignedChild */ SELECT 1 FROM ${config.child.table}`);
}

function* yieldedChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  yield config.child;
  return query(rawSql`/* yieldedChild */ SELECT 1 FROM ${config.child.table}`);
}

export function resumeYield() {
  const iterator = yieldedChild("user");
  const child = iterator.next().value as any;
  child.table = unknownTable();
  iterator.next();
}

export function caughtChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  try {
    throw config.child;
  } catch (error) {
    escape(error);
  }
  return query(rawSql`/* caughtChild */ SELECT 1 FROM ${config.child.table}`);
}

export function wrappedChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  const wrapped = { child: config.child };
  escape(wrapped);
  return query(rawSql`/* wrappedChild */ SELECT 1 FROM ${config.child.table}`);
}

export function iteratedChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  for (const child of [config.child]) escape(child);
  return query(rawSql`/* iteratedChild */ SELECT 1 FROM ${config.child.table}`);
}

export function classChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  class Holder {
    child = config.child;
  }
  escape(new Holder());
  return query(rawSql`/* classChild */ SELECT 1 FROM ${config.child.table}`);
}

export function defaultChild(kind: "user" | "post") {
  const CONFIG = {
    user: { child: { table: "users" } },
    post: { child: { table: "posts" } },
  } as const;
  const config = CONFIG[kind];
  function expose(child = config.child) {
    escape(child);
  }
  expose();
  return query(rawSql`/* defaultChild */ SELECT 1 FROM ${config.child.table}`);
}
