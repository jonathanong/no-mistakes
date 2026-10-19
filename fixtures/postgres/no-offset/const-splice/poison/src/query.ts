import { read } from "@example/db";

const SQL = { t: "orders" };

export function list() {
  return read(`SELECT id FROM ${SQL.t} OFFSET 10`);
}

export function write() {
  SQL.t = "x";
}
