import { query } from "@example/db";

export function list() {
  return query(`SELECT id FROM users WHERE tenant = ${process.env.SCHEMA}`);
}
