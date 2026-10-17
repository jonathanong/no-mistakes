import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql, visibleTierSql } from "./helpers/index.mts";

// `ownerVisibleSql` crosses two `export *` hops; `visibleTierSql` is a renamed
// named re-export. Both must resolve to the declaring module.
export async function listLinks(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.owner_id"));
  query.append(sql` AND `);
  query.append(visibleTierSql());
  return read(query);
}
