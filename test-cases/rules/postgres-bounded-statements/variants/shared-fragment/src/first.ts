import { query, sql } from "@example/db";
import { sharedFragmentSql } from "./fragment";

export function first() {
  const statement = sql`SELECT a.id FROM accounts a`.append(sharedFragmentSql());
  return query(statement);
}
