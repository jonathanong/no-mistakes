import { read } from "@example/db";
const TABLES = { orders: "orders" };
const OTHER = { accounts: "accounts" };
export function mismatches(kind: string) {
  read(`SELECT ${TABLES[kind as keyof typeof OTHER]}`);
  read(`SELECT ${TABLES[kind as string]}`);
  read(`SELECT ${TABLES[kind as keyof Other]}`);
  read(`SELECT ${TABLES[kind as typeof TABLES]}`);
}
type Other = { accounts: string };
