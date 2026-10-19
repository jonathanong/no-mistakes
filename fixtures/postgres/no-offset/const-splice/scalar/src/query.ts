import { read } from "@example/db";

const PAGE = "OFFSET 10";
const TABLES = { page: "OFFSET 10" };

export function scalar() {
  return read(`SELECT id FROM orders ${PAGE}`);
}

export function member() {
  return read(`SELECT id FROM orders ${TABLES.page}`);
}
