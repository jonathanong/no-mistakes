import sql from "sql-template-strings";
import { read } from "@example/db";

// Judged rewrite of an executor adapter: the helper names `read` on the SQL text.
function run(a: string) {
  return read(sql`SELECT id FROM orders WHERE account_id = ${a} OFFSET 1`); // findings: offset
}

export function list(a: string) {
  return run(a);
}
