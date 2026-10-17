import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "./helper.mts";

export async function atExecutor() {
  // no-mistakes-disable-next-line postgres-required-predicates
  return read(ownerVisibleSql("l.owner_id"));
}

export async function atExecutorOfBuilder(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.owner_id"));
  // no-mistakes-disable-next-line postgres-required-predicates
  return read(query);
}

export async function atHelperCall(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  // no-mistakes-disable-next-line postgres-required-predicates
  query.append(ownerVisibleSql("l.owner_id"));
  return read(query);
}

// Nothing opts out here, so the helper's line is reported.
export async function unsuppressed(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.owner_id"));
  return read(query);
}
