import { query } from "@example/db";

export function list() {
  // no-mistakes-disable-next-line postgres-no-unbounded-sql-identifier
  return query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}
