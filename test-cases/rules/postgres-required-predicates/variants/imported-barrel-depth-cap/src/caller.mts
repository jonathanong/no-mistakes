import sql from "sql-template-strings";
import { read } from "@example/db";
import { visibleSql } from "./chain/barrel-1.mts";

// Crosses 8 `export *` re-exports before reaching the declaring module.
export async function listLinks(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.owner_id"));
  return read(query);
}
