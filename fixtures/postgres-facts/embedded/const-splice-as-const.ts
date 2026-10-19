import { read } from "@example/db";

const tableName = "orders" as const;

export function list(accountId: string) {
  return read(`SELECT id FROM ${tableName} WHERE account_id = $1`, [accountId]);
}
