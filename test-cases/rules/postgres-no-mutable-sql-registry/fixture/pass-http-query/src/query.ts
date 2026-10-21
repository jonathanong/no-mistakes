import { query } from "@example/db";
import { tables } from "./tables";

export function search(req: { query: (sql: string) => unknown }) {
  return req.query(`SELECT id FROM ${tables.users}`);
}

export function list() {
  return query(`SELECT id FROM users`);
}
