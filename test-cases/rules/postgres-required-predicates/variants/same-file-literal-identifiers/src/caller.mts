import sql from "sql-template-strings";
import { read } from "@example/db";

function ownerVisibleSql(parentCol: string, ownerCol: string): string {
  return `(${parentCol} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol}))`;
}

const sameColumn = (column: string) =>
  `EXISTS (SELECT 1 FROM view_active_plans q WHERE q.account_id = ${column} AND ${column} IS NOT NULL)`;

// A same-file helper's untagged template interpolates caller-supplied literals.
export async function withLiterals(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.parent_id", "l.account_id"));
  return read(query);
}

export async function withRepeatedParameter(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(sameColumn("l.owner_id"));
  return read(query);
}

// An optional call may not run the helper at all.
export async function withOptionalCall(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql?.("l.parent_id", "l.account_id"));
  return read(query);
}

// A non-literal argument still cannot be spliced.
export async function withVariable(id: string, column: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql(column, "l.account_id"));
  return read(query);
}

// A no-substitution template is the same literal as a quoted string.
export async function withBacktick(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql(`l.parent_id`, `l.account_id`));
  return read(query);
}
