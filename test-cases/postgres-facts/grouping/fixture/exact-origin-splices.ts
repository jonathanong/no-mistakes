import { query, sql } from "db";
const table = "posts"; query(`SELECT DISTINCT id FROM ${table}`);
const longer = "SELECT DISTINCT id, author_id, created_at, parent_id FROM posts WHERE parent_id IS NOT NULL";
query(`${longer} ORDER BY id`);
const typed = sql`SELECT DISTINCT id FROM posts WHERE author_id = ${author}`;
query(sql`${typed} ORDER BY id`);
export function opaque() {
  query(`SELECT DISTINCT id FROM posts ${unresolved()}`);
}
