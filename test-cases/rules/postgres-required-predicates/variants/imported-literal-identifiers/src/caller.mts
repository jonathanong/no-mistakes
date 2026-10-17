import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "./visibility-sql.mts";

// The helper's template reads the guarded view; its text sits between two
// caller fragments, so every physical line must still map back correctly.
export async function listLinks(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.parent_id", "l.account_id"));
  query.append(sql` ORDER BY l.id`);
  return read(query);
}
