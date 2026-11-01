import { query } from "@example/db";

declare function assertWhitelistedSqlIdentifier(
  value: string,
  allowed: ReadonlySet<string>,
  name: string,
): void;

const ALLOWED = new Set(["users", "posts"]);

export function list(table: string) {
  assertWhitelistedSqlIdentifier(table, ALLOWED, "table");
  return query(`SELECT id FROM ${table}`);
}
