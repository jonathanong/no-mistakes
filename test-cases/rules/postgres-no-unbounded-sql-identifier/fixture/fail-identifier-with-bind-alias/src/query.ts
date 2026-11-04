import { query } from "@example/db";

export function single(table: string, value: string) {
  const alias = value;
  return query(`SELECT id FROM ${table} WHERE name = ${alias}`);
}

export function chained(table: string, value: string) {
  const first = value;
  const second = first;
  return query(`SELECT id FROM ${table} WHERE name = ${second}`);
}
