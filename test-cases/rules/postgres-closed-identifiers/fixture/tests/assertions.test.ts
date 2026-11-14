import { query, sql } from "@example/db";
import { assertWhitelistedSqlIdentifier as allowed } from "@example/identifiers";

export function inlineSet(table: string) {
  const safe = allowed(table, new Set(["users", "posts"]));
  return query(sql`/* inlineSet */ SELECT 1 FROM `.append(safe).append(sql` WHERE id = $1`));
}

export function inlineArray(table: string) {
  return query(
    sql`/* inlineArray */ SELECT 1 FROM `
      .append(allowed(table, ["users", "posts"]))
      .append(sql` WHERE id = $1`),
  );
}

export function localAllowlist(table: string) {
  const names = new Set(["users", "posts"]);
  return query(
    sql`/* localAllowlist */ SELECT 1 FROM `
      .append(allowed(table, names))
      .append(sql` WHERE id = $1`),
  );
}

function assertSafeSqlIdentifier(value: string) {
  const names = new Set(["users", "posts"]);
  if (!names.has(value)) throw new Error("unknown table");
  return value;
}

export function helperOwnedAllowlist(table: string) {
  return query(
    sql`/* helperOwnedAllowlist */ SELECT 1 FROM `
      .append(assertSafeSqlIdentifier(table))
      .append(sql` WHERE id = $1`),
  );
}
