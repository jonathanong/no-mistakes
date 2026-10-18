import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";
import { read } from "@example/db";
import { ownerClause } from "./owner-clause.mts";

// A fluent statement appends every link to the tracked builder, in order.
// The guarded view is read only in the second link.
export async function secondLinkReadsTheView(id: string) {
  const query = sql`SELECT r.id FROM roots r WHERE r.owner = ${id} AND `;
  query.append(sql`TRUE`).append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
  return read(query);
}

// The first link is an imported helper, which is a hole filled later.
export async function helperThenView(id: string) {
  const query = sql`SELECT r.id FROM roots r WHERE r.owner = ${id} AND `;
  query.append(ownerClause("r.parent_id")).append(sql` AND EXISTS (SELECT 1 FROM view_active_plans w WHERE w.tier = 'b')`);
  return read(query);
}

// A builder bound to a name and appended into a fluent expression is copied
// at that moment.
export async function boundThenChained(id: string) {
  const inner = sql`EXISTS (SELECT 1 FROM view_active_plans x WHERE x.tier = 'c')`;
  const query = sql`SELECT r.id FROM roots r WHERE `.append(inner).append(sql` AND r.owner = ${id}`);
  return read(query);
}

// A builder the walk cannot track (a parameter) has no known prefix, so a
// chain on it recovers nothing.
export async function untrackedRoot(query: SQLStatement) {
  query.append(sql`TRUE`).append(sql` AND EXISTS (SELECT 1 FROM view_active_plans y WHERE y.tier = 'd')`);
  return read(query);
}
