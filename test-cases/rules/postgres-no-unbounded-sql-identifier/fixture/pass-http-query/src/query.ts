import { query } from "@example/db";

export function search(req: { query: (sql: string) => unknown }) {
  return req.query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}

export function list() {
  return query(`SELECT id FROM users`);
}
