import { query } from "@example/db";
import { sql } from "untrusted-tag-library";

function build(id: number) {
  return sql`SELECT * FROM topics WHERE id = ${id}`;
}

export function run() {
  return query(build(1));
}
