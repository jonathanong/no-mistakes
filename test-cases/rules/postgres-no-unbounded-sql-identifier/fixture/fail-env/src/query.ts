import { query } from "@example/db";

export function list() {
  return query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}
