import { query, sql } from "@example/db";
import { sharedFragmentSql } from "./fragment";

export function suppressed() {
  const statement = sql`SELECT a.id FROM accounts a`;
  statement.append(sharedFragmentSql());
  // no-mistakes-disable-next-line postgres-bounded-statements
  return query(statement);
}
