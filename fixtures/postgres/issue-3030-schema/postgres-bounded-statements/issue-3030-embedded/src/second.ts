import { query, sql } from "@example/db";
import { rootsSql } from "./shared";

export function second() {
  const statement = sql`SELECT a.id, o.id `.append(rootsSql());
  return query(statement);
}
