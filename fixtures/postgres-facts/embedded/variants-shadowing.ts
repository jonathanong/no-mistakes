import { query, sql } from "@example/db";
const fragment = sql`WHERE id = ${id}`;
const alias = fragment;
query(sql`SELECT id FROM users ${alias}`);
{
  const alias = "ordinary bind value";
  query(sql`SELECT id FROM users WHERE name = ${alias}`);
}
function untrusted(sql) {
  query(sql`SELECT id FROM users ${fragment}`);
}
function local() {
  const fragment = sql`WHERE active`;
  query(sql`SELECT id FROM users ${fragment}`);
}
