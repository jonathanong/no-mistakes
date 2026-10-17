import sql from "untrusted-tag-library";
import { query } from "@example/db";

export function load(id: number) {
  return query(sql`SELECT * FROM topics WHERE id = ${id}`);
}
