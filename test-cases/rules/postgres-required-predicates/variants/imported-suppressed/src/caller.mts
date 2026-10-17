import sql from "sql-template-strings";
import { read } from "@example/db";
import { reviewedSql, unreviewedSql } from "./visibility-sql.mts";

// Findings land on the helper module's lines, so a directive there applies.
export async function reviewed(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(reviewedSql("l.owner_id"));
  return read(query);
}

export async function unreviewed(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(unreviewedSql("l.owner_id"));
  return read(query);
}
