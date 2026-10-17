// no-mistakes-disable-file postgres-required-predicates
import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "./helper.mts";

// The caller opts out of the whole file, so SQL it executes is not reported
// even though the reported line would be in the helper's module.
export async function direct() {
  return read(ownerVisibleSql("l.owner_id"));
}

export async function appended(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.owner_id"));
  return read(query);
}
