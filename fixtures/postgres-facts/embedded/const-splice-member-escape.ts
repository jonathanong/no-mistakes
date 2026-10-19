import { read } from "@example/db";

const TABLES = { users: "users" };

function hide(value: { users: string }) {
  return value;
}

// Passing the object is an escape, so the member read stays unknown.
export function list() {
  hide(TABLES);
  return read(`SELECT id FROM ${TABLES.users} WHERE account_id = $1`);
}
