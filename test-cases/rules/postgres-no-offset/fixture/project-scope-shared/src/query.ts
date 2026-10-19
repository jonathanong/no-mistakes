import { query } from "@example/db";

export function page() {
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}
