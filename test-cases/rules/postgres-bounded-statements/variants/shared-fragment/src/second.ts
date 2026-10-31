import { query, sql } from "@example/db";
import { sharedFragmentSql } from "./fragment";

export function second() {
  const statement = sql`SELECT a.id FROM accounts a`;
  statement.append(sharedFragmentSql());
  return query(statement);
}
