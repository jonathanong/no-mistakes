import { query } from "@example/db";
import { sql } from "@example/sql";
import legacy from "sql-template-strings";

export function fragments(value: string, n: number) {
  const fragment = sql`id FROM records WHERE value = ${value}`;
  query(`SELECT ${fragment.text}`, fragment.values);
  const relation = sql`records WHERE value = ${value}`;
  query(`SELECT id FROM ${relation.text}`, relation.values);
  const oldFragment = legacy`id FROM records WHERE value = ${value}`;
  query(`SELECT ${oldFragment.text}`, oldFragment.values);
}
