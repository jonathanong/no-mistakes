import { query } from "@example/db";

export function identifier(table: string, value: string) {
  const name = table;
  const bind = value;
  return query(`SELECT id FROM ${name} WHERE name = ${bind}`);
}

export function environment(table: string) {
  const first = process.env.COLUMN;
  const second = first;
  return query(`SELECT id FROM ${table} WHERE ${second} = 1`);
}

export function reassigned(table: string, value: string) {
  let alias = value;
  alias = process.env.COLUMN;
  return query(`SELECT id FROM ${table} WHERE ${alias} = 1`);
}

export function unknown(table: string, value: string) {
  const alias = normalize(value);
  return query(`SELECT id FROM ${table} WHERE ${alias} = 1`);
}
