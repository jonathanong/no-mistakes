import { read } from "@example/db";

export const TABLES = { users: "users" };

export function list(accountId: string) {
  return read(`SELECT id FROM ${TABLES.users} WHERE account_id = $1`, [accountId]);
}
