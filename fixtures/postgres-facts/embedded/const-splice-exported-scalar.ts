import { read } from "@example/db";

export const tableName = "orders";

export function list(accountId: string) {
  return read(`SELECT id FROM ${tableName} WHERE account_id = $1`, [accountId]);
}
