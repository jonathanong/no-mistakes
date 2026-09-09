import { query } from "@data-stores/psql";

function sql() {
  return process.env.SQL;
}

function build() {
  return sql`SELECT 1`;
}

export function load() {
  return query(build());
}
