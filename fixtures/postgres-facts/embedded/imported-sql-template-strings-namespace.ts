import * as sql from "sql-template-strings";
import { query } from "@data-stores/psql";

// Namespace import binds the module object, not the tag function.
function build(id: number) {
  return sql`SELECT * FROM topics WHERE id = ${id}`;
}

export function run() {
  return query(build(1));
}
