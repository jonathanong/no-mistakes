import { read } from "@example/db";

const TABLES = { orders: "orders", accounts: "accounts" };
const QUOTED = { "order items": "order items" };

export function byKind(kind: string, flag: boolean, id: string) {
  read(`SELECT id FROM ${TABLES[kind]} WHERE id = $1`, [id]);
  read(TABLES?.[kind]);
  read(sql`SELECT id FROM ${QUOTED[flag ? "order items" : "other"]}`);
  read(`SELECT ${TABLES[`orders`]}`);
  read(`SELECT ${TABLES.orders}`);
}

declare function sql(text: TemplateStringsArray, ...values: unknown[]): string;
