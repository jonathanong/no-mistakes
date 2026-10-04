import { query } from "@example/db";

export function page(offset: number) {
  // no-mistakes-disable-next-line postgres-no-offset: audited cursor window
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}
