import sql from "untrusted-tag-library";
import { query } from "@data-stores/psql";

export function load(id: number) {
  return query(sql`SELECT * FROM topics WHERE id = ${id}`);
}
