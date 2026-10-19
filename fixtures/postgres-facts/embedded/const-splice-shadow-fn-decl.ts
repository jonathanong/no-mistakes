import { read } from "@example/db";

const PAGE = "orders";

export function list() {
  const result = read(`SELECT id FROM ${PAGE} WHERE account_id = $1`);
  function PAGE() {
    return "";
  }
  return PAGE(), result;
}
