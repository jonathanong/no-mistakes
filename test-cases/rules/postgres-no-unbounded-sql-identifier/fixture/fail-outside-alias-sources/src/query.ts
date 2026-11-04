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

export function member(table: string, value: { column: string }) {
  const alias = value;
  return query(`SELECT id FROM ${table} WHERE ${alias.column} = 1`);
}

export function memberSource(table: string, value: { column: string }) {
  const alias = value.column;
  return query(`SELECT id FROM ${table} WHERE ${alias} = 1`);
}

export function parameterWrite(table: string, value: string) {
  value = process.env.COLUMN;
  const alias = value;
  return query(`SELECT id FROM ${table} WHERE ${alias} = 1`);
}

export function destructured(table: string, { column }: { column: string }) {
  const alias = column;
  return query(`SELECT id FROM ${table} WHERE ${alias} = 1`);
}

export function defaultSource(table: string, value = process.env.COLUMN) {
  const alias = value;
  return query(`SELECT id FROM ${table} WHERE ${alias} = 1`);
}
