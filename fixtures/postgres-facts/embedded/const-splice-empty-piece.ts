import { read } from "@example/db";

export function list(accountId: string) {
  return read(`SELECT id FROM ${``}orders WHERE account_id = $1`, [accountId]);
}
