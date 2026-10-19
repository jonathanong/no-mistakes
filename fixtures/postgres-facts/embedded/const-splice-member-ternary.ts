import { read } from "@example/db";

const SQL = { page: "OFFSET 10" };

export function forward(c: boolean) {
  return read(c ? `SELECT id FROM orders ${SQL.page}` : `SELECT id FROM orders`);
}

export function swapped(c: boolean) {
  return read(c ? `SELECT id FROM orders` : `SELECT id FROM orders ${SQL.page}`);
}
