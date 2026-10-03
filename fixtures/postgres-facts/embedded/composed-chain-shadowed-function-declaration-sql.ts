import { query } from "@example/db";

function sql() {
  return "DELETE FROM users";
}

function build() {
  return sql`SELECT 1`;
}

export function load() {
  return query(build());
}
