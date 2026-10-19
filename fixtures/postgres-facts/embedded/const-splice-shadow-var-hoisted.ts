import { read } from "@example/db";

const PAGE = "orders";

export function list(x: string) {
  const result = read(`SELECT id FROM ${PAGE} WHERE account_id = $1`);
  var PAGE = x;
  return result;
}
