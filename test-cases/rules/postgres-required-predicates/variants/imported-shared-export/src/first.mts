import sql from "sql-template-strings";
import { read } from "@example/db";
import { visibleSql } from "./index.mts";

// Three call sites in two files import the same export through one barrel.
export async function byOwner(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.owner_id"));
  return read(query);
}

export async function byParent(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.parent_id"));
  return read(query);
}
