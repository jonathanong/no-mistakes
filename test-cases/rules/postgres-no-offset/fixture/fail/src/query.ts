import { query } from "@data-stores/psql";

export function page(offset: number) {
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}
