import { read } from "@example/db";

const TABLES = { orders: "orders", accounts: "accounts" };
let mutable = { orders: "orders" };
const spread = { ...TABLES };
const shorthand = { TABLES };
const method = { orders() { return "orders"; } };
const computed = { [TABLES.orders]: "orders" };
const tagged = { orders: sql`SELECT 1` };
const empty = {};
const alias = TABLES;

export function silent(kind: string, TABLES: Record<string, string>, id: string) {
  read(`SELECT ${TABLES["orders"]}`);
  read(`SELECT ${TABLES[1]}`);
  read(`SELECT ${TABLES[`a${"b"}`]}`);
  read(`SELECT ${TABLES.orders}`);
  read(`SELECT ${mutable[kind]}`);
  read(`SELECT ${spread[kind]}`);
  read(`SELECT ${shorthand[kind]}`);
  read(`SELECT ${method[kind]}`);
  read(`SELECT ${computed[kind]}`);
  read(`SELECT ${tagged[kind]}`);
  read(`SELECT ${empty[kind]}`);
  read(`SELECT ${alias[kind]}`);
  read(`SELECT ${missing[kind]}`);
  read(`SELECT ${{ orders: "orders" }[kind]}`);
  read(...[`SELECT ${TABLES[kind]}`]);
  read(`SELECT id FROM orders`, [TABLES[kind]]);
  console.log(`SELECT ${TABLES[kind]}`);
  read(`SELECT ${TABLES[kind]}`);
}

declare function sql(text: TemplateStringsArray, ...values: unknown[]): string;
declare const missing: Record<string, string>;
