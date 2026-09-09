import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

function build(id: number) {
  return sql`SELECT * FROM topics WHERE id = ${id}`;
}

export function run() {
  return query(build(1));
}
