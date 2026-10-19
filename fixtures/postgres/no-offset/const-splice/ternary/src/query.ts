import { read } from "@example/db";

const SQL = { page: "OFFSET 10" };

export function list(c: boolean) {
  return read(c ? `SELECT id FROM orders ${SQL.page}` : `SELECT id FROM orders`);
}
