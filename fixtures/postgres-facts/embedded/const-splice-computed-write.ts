import { read } from "@example/db";

const TABLES = { users: "users" };

export function list() {
  TABLES["users"] = "accounts";
  return read(`SELECT id FROM ${TABLES.users} WHERE account_id = $1`);
}
