import sql from "sql-template-strings";
import { read } from "@example/db";
import { visibleSql } from "./index.mts";

export async function byCreator(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.creator_id"));
  return read(query);
}
