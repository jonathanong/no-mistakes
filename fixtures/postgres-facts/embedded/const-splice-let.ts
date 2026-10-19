import { read } from "@example/db";

let tableName = "orders";

export function list() {
  return read(`SELECT id FROM ${tableName} WHERE account_id = $1`);
}
