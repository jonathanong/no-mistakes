import sql from "sql-template-strings";
import { read } from "@example/db";

const SQL = { page: " OFFSET 10", none: "" };

export function oneArm(c: boolean) {
  const query = sql`SELECT id FROM orders`;
  if (c) query.append(SQL.page);
  return read(query);
}

export function bothArms(c: boolean) {
  const query = sql`SELECT id FROM orders`;
  if (c) query.append(SQL.none);
  else query.append(SQL.page);
  return read(query);
}
