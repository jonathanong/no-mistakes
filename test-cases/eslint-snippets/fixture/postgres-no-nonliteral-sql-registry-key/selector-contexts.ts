import { read } from "@example/db";
const TABLES = { orders: "orders", accounts: "accounts" };
declare function choose(): Promise<string>;

export async function awaited() {
  read(`SELECT ${TABLES[await choose()]}`);
}
export function* yielded() {
  read(`SELECT ${TABLES[yield "key"]}`);
}
declare function chooseSync(): void;
export function sequence(kind: string) {
  // Keep the comma expression as one argument and evaluate its side effect once.
  read(`SELECT ${TABLES[(chooseSync(), kind)]}`);
}
