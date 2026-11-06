import { query, sql } from "@example/db";
import { rootsSql } from "./shared";

export function suppressed() {
  const statement = sql`SELECT a.id, o.id `.append(rootsSql());
  // no-mistakes-disable-next-line postgres-bounded-statements
  return query(statement);
}
