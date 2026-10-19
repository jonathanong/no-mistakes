import { read } from "@example/db";

const base = { users: "users" };
const TABLES = { ...base, extra: "extra" };

export function list() {
  return read(`SELECT id FROM ${TABLES.users} WHERE account_id = $1`);
}
