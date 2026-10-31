import { query, sql } from "@example/db";
import { pairFragmentSql } from "./pair";

export function third() {
  const statement = sql`SELECT o.id FROM`;
  statement.append(pairFragmentSql());
  return query(statement);
}
