import { query, read, sql } from "@example/db";
import { it } from "vitest";

it.each(["orders", "accounts"])("rows", (key) => {
  const tables = {
    orders: { sql: "orders" },
    accounts: { sql: "accounts" },
  };
  const chosen = tables[key];
  read(`SELECT ${chosen.sql}`);
});

it.each(["orders"])("numeric", (key) => {
  const tables = { orders: { sql: "SELECT orders" } };
  const chosen = tables[1];
  void key;
  void chosen;
  query(sql`SELECT 1`);
});

it.each(...["orders"])("spread", () => {
  query(sql`SELECT 1`);
});

const more: string[] = [];
it.each(["orders", ...more])("element", () => {
  query(sql`SELECT 1`);
});

it.each([1])("number", () => {
  query(sql`SELECT 1`);
});
