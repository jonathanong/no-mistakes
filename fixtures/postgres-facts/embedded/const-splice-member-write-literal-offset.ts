import { read } from "@example/db";

const SQL = { t: "orders", n: "2" };

export function table() {
  const result = read(`SELECT id FROM ${SQL.t} OFFSET 10`);
  SQL.t = "x";
  return result;
}

export function numeric() {
  const result = read(`SELECT id FROM orders OFFSET ${SQL.n}`);
  SQL.n = "9";
  return result;
}
