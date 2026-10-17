import sql from "sql-template-strings";
import { read } from "@example/db";
import { pageClause } from "./helper.mts";

export async function atExecutor() {
  // no-mistakes-disable-next-line postgres-no-offset
  return read(pageClause("20"));
}

export async function atExecutorOfBuilder(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  query.append(pageClause("20"));
  // no-mistakes-disable-next-line postgres-no-offset
  return read(query);
}

export async function atHelperCall(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  // no-mistakes-disable-next-line postgres-no-offset
  query.append(pageClause("20"));
  return read(query);
}

// Nothing opts out here, so the helper's line is reported.
export async function unsuppressed(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  query.append(pageClause("20"));
  return read(query);
}
