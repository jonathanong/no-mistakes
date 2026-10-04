import { query } from "@example/db";

export function page(offset: number) {
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}
