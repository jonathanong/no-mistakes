import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "./visibility-sql.mts";

// Literal substitution needs every argument to be a string literal at the
// call site, so each query below must stay unanalyzable.
export async function fromParameter(id: string, column: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql(column, "l.account_id"));
  return read(query);
}

const ownerColumn = "l.account_id";

export async function fromConstant(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.parent_id", ownerColumn));
  return read(query);
}

export async function tooFewArguments(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.parent_id"));
  return read(query);
}

export async function spreadArguments(id: string, columns: [string, string]) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql(...columns));
  return read(query);
}

// A no-substitution template literal is not a string literal on this path either.
export async function fromBacktick(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql(`l.parent_id`, `l.account_id`));
  return read(query);
}
