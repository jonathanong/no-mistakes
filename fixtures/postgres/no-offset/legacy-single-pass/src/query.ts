import { query } from "@example/db";

export function firstPage(limit: number) {
  return query(`SELECT id FROM posts ORDER BY id DESC LIMIT ${limit + 1}`);
}
