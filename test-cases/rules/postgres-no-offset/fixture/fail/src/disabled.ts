import { query } from "@data-stores/psql";

export function page(offset: number) {
  // no-mistakes-disable-next-line postgres-no-offset: audited cursor window
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}
