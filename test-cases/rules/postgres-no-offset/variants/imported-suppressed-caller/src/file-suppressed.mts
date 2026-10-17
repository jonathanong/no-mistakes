// no-mistakes-disable-file postgres-no-offset
import sql from "sql-template-strings";
import { read } from "@example/db";
import { pageClause } from "./helper.mts";

export async function direct() {
  return read(pageClause("20"));
}

export async function appended(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  query.append(pageClause("20"));
  return read(query);
}
