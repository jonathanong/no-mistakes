import { query } from "@example/db";

export function list() {
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
