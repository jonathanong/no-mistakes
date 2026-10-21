import { read } from "@example/db";
// Duplicate keys use the final value, never an unreachable second case.
const TABLES = { orders: "old", accounts: "{{accounts}}", orders: 'order "items"' };
export function byKind(kind: string) {
  read(`SELECT ${TABLES[kind]}`);
}
