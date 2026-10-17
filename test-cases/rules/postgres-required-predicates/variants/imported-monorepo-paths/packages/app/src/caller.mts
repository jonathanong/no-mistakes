import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "@/visibility.mts";

export async function listLinks(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.owner_id"));
  return read(query);
}
