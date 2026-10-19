import { read } from "@example/db";

const TABLES = { users: "users" };

// A later write reaches the earlier splice. const does not freeze members.
export function writeAfterRead() {
  const result = read(`SELECT id FROM ${TABLES.users} WHERE account_id = $1`);
  TABLES.users = "accounts";
  return result;
}

export function writeBeforeRead() {
  TABLES.users = "accounts";
  return read(`SELECT id FROM ${TABLES.users} WHERE account_id = $1`);
}
