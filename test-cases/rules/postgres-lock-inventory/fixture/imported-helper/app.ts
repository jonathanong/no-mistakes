import sql from "sql-template-strings";
import { query } from "@example/db";
import { statement } from "./statement";

export function first() {
  const prepared = sql``;
  prepared.append(statement);
  return query(prepared);
}

export function second() {
  const prepared = sql``;
  prepared.append(statement);
  // no-mistakes-disable-next-line postgres-lock-inventory
  return query(prepared);
}
