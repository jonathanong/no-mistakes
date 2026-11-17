import { query, sql } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";
import { assertWhitelistedSqlIdentifier as allowed } from "@example/identifiers";

declare function escape(value: unknown): void;

export function typedDrop(table: "users" | "posts") {
  return query(sql`/* typedDrop */ DROP TABLE `.append(table));
}

export function assertedDrop(table: string) {
  return query(
    sql`/* assertedDrop */ DROP TABLE `.append(allowed(table, new Set(["users", "posts"]))),
  );
}

function command(kind: "read" | "mutate") {
  switch (kind) {
    case "read":
      return "SELECT 1";
    case "mutate":
      return "DROP TABLE posts";
  }
}

export function switchMutation(kind: "read" | "mutate") {
  return query(sql`/* switchMutation */ `.append(command(kind)));
}

export function rawFunction(expression: "1" | "pg_sleep(1)") {
  return query(rawSql`/* rawFunction */ SELECT ${expression}`);
}

export function unboundedColumn(column: "user_id" | "post_id") {
  return query(rawSql`/* unboundedColumn */ UPDATE users SET status = $1 WHERE ${column} = $2`);
}

export function unboundedTables(table: "users" | "posts") {
  return query(rawSql`/* unboundedTables */ SELECT 1 FROM ${table}`);
}

const frozenCommands = Object.freeze({
  keep: "SELECT 1",
  drop: "DROP TABLE posts", // frozenDrop
} as const);

function frozenCommand(kind: string) {
  if (!Object.hasOwn(frozenCommands, kind)) throw new Error(`unknown ${kind}`);
  return frozenCommands[kind as keyof typeof frozenCommands]; // frozenDrop
}

export function frozenDrop(kind: string) {
  return query(sql`/* frozenDrop */ `.append(frozenCommand(kind)));
}

export function suppressedDrop(table: "users" | "posts") {
  // no-mistakes-disable-next-line postgres-no-test-state-mutation
  return query(sql`/* suppressedDrop */ DROP TABLE `.append(table));
}

export function mixedPoisoned(kind: "user" | "post", mutate: boolean) {
  const CONFIG = { user: { table: "users" }, post: { table: "posts" } } as const;
  escape(CONFIG);
  const mixedPoisonedStatement = mutate
    ? sql`/* mixedPoisoned */ DROP TABLE users`
    : rawSql`SELECT 1 FROM ${CONFIG[kind].table}`;
  return query(mixedPoisonedStatement);
}
