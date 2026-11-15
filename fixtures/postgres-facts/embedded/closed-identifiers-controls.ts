import { query } from "@example/db";
import { assertIdentifier as safe, assertAtTwo } from "@example/identifiers";
import { wrong as wrongAssertion } from "@example/identifiers";
import type { assertIdentifier as typeAssertion } from "@example/identifiers";
import defaultAssertion from "@example/identifiers";
import * as assertionNamespace from "@example/identifiers";
import { sql as raw } from "@example/raw-sql";
import { sql as wrongModule } from "@example/other-sql";
import { wrong as wrongRaw } from "@example/raw-sql";
import type { sql as typeRaw } from "@example/raw-sql";
import * as rawNamespace from "@example/raw-sql";
import defaultRaw from "@example/raw-sql";

type Alias = "users" | "posts";
export function allowlists(table: string, list: string[]) {
  query(raw`LOCK TABLE ${safe(table)}`);
  query(raw`LOCK TABLE ${assertAtTwo(table, ["users", "posts"])}`);
  query(raw`LOCK TABLE ${safe(table, ...list)}`);
  query(raw`LOCK TABLE ${safe(table, [])}`);
  query(raw`LOCK TABLE ${safe(table, new Set())}`);
  query(raw`LOCK TABLE ${safe(table, ["users", 7])}`);
}
export function shadowSet(table: string, Set: any) {
  query(raw`LOCK TABLE ${safe(table, new Set(["users", "posts"]))}`);
}
export function optional(table?: "users" | "posts") {
  query(raw`LOCK TABLE ${table}`);
}
export function defaults(table: "users" | "posts" = runtime()) {
  query(raw`LOCK TABLE ${table}`);
}
export function widened(table: string) {
  query(raw`LOCK TABLE ${table}`);
}
export function nonstring(table: "users" | 7) {
  query(raw`LOCK TABLE ${table}`);
}
export function alias(table: Alias) {
  query(raw`LOCK TABLE ${table}`);
}
export function casts(table: string) {
  const cast = table as "users" | "posts";
  query(raw`LOCK TABLE ${cast}`);
  const checked = table satisfies "users" | "posts";
  query(raw`LOCK TABLE ${checked}`);
}
export function writer(table: "users" | "posts") {
  function mutate() {
    table = runtime();
  }
  mutate();
  query(raw`LOCK TABLE ${table}`);
}
export function records(kind: "user" | "post") {
  const getter = {
    user: {
      get table() {
        return "users";
      },
    },
    post: { table: "posts" },
  } as const;
  query(raw`LOCK TABLE ${getter[kind].table}`);
  const computed = { ["user"]: { table: "users" }, post: { table: "posts" } } as const;
  query(raw`LOCK TABLE ${computed[kind].table}`);
  const spread = { ...runtime(), user: { table: "users" }, post: { table: "posts" } } as const;
  query(raw`LOCK TABLE ${spread[kind].table}`);
  const prototype = { __proto__: { user: { table: "users" } }, post: { table: "posts" } } as const;
  query(raw`LOCK TABLE ${prototype[kind].table}`);
}
export function imports(table: string) {
  query(raw`LOCK TABLE ${wrongAssertion(table, ["users", "posts"])}`);
  query(raw`LOCK TABLE ${typeAssertion(table, ["users", "posts"])}`);
  query(raw`LOCK TABLE ${defaultAssertion(table, ["users", "posts"])}`);
  query(raw`LOCK TABLE ${assertionNamespace.assertIdentifier(table, ["users", "posts"])}`);
  query(wrongModule`LOCK TABLE ${table}`);
  query(wrongRaw`LOCK TABLE ${table}`);
  query(typeRaw`LOCK TABLE ${table}`);
  query(rawNamespace.sql`LOCK TABLE ${table}`);
  query(defaultRaw`LOCK TABLE ${table}`);
}
export function shadowRaw(raw: any, table: string) {
  query(raw`LOCK TABLE ${table}`);
}
