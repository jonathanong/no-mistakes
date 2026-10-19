import sql from "sql-template-strings";
import { read } from "@example/db";

function run(exec: (statement: unknown) => unknown, account: string) {
  return exec(sql`SELECT id FROM orders WHERE account_id = ${account} OFFSET 1`);
}

export function list(account: string) {
  return run(read, account);
}
