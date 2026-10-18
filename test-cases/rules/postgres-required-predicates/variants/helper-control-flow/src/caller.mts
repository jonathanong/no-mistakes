import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";
import { read } from "@example/db";
import { appendListQuery } from "./list-query.mts";
import { appendTier } from "./decided.mts";

// The helper returns early on one branch, so there are two alternatives and
// both read the guarded view by a non-key column.
export async function listing(all: boolean, limit: number) {
  const query = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = 'a'`;
  appendListQuery(query, { all, limit });
  return read(query);
}

// A same-file typed appender with the same early-return shape.
function appendWindow(query: SQLStatement, options: { all: boolean; limit: number }) {
  if (options.all) {
    query.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans w WHERE w.tier = 'window')`);
    return;
  }
  query.append(sql` LIMIT ${options.limit}`);
}

export async function sameFile(all: boolean, limit: number) {
  const query = sql`SELECT r.id FROM roots r WHERE r.kind = 'x'`;
  appendWindow(query, { all, limit });
  return read(query);
}

// A literal `true` decides the helper's test: only the strict arm can run.
export async function decidedStrict() {
  const query = sql`SELECT r.id FROM roots r WHERE r.kind = 'x'`;
  appendTier(query, true);
  return read(query);
}

// A value-position call must return the very builder it was given. This
// helper returns a flag, so combining its side effect with its result stays
// opaque.
function appendAndReport(query: SQLStatement, flag: boolean): boolean {
  if (flag) query.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans g WHERE g.tier = 'g')`);
  return flag;
}

export async function resultUsedAsValue(flag: boolean) {
  const query = sql`SELECT r.id FROM roots r WHERE r.kind = 'x'`;
  return read(appendAndReport(query, flag) ? query : sql`SELECT 1`);
}
