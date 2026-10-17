import sql from "sql-template-strings";
import { read } from "@example/db";
import { visibleSql } from "./barrel-a.mts";

// The two barrels re-export each other and nothing declares `visibleSql`,
// so resolution must stop at the cycle instead of looping or guessing.
export async function listLinks(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.owner_id"));
  return read(query);
}
