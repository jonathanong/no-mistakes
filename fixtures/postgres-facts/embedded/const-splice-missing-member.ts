import { read } from "@example/db";

const TABLES = { users: "users" };

export function list(accountId: string) {
  return read(`SELECT id FROM ${TABLES.orders} WHERE account_id = $1`, [accountId]);
}
