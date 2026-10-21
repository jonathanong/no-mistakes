import { read } from "@example/db";
const TABLES = { orders: "orders" };
export function unchanged(kind: string, argument: string) {
  const literal = `SELECT ${TABLES["orders"]}`;
  read(literal);
  let mutable = `SELECT ${TABLES[kind]}`;
  read(mutable);
  read(argument);
  read(missing);
}
declare const missing: string;
