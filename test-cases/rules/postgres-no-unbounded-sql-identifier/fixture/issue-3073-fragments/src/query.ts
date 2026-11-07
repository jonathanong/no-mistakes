import { query } from "@example/db";
import { sql } from "@example/sql";
import legacy from "sql-template-strings";

export function fragments(value: string, n: number) {
  const fragment = sql`id FROM records WHERE value = ${value}`;
  query(`SELECT ${fragment.text}`, fragment.values);
  const oldFragment = legacy`id FROM records WHERE value = ${value}`;
  query(`SELECT ${oldFragment.text}`, oldFragment.values);
  const conditions = [`id = $${n}`, "active = $" + (n + 1)];
  const where = conditions.join(" AND ");
  query(`SELECT id FROM records WHERE ${where}`);
}

export function unchecked(table: "users" | "posts") {
  query(`SELECT id FROM ${table}`);
}
