import { query } from "@example/db";

const ROWS = [
  { key: "user", schema: "public", table: "users" },
  { key: "post", schema: "archive", table: "posts" },
] as const;

function getRowOrThrow(key: string) {
  const selected = ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}

export async function literalRow() {
  const selected = getRowOrThrow("user");
  await query(`SELECT * FROM ${selected.schema}.${selected.table}`);
}

export async function dynamicRow(key: string) {
  const selected = getRowOrThrow(key);
  await query(`SELECT * FROM ${selected.table}`);
}

export async function directRow() {
  await query(`SELECT * FROM ${getRowOrThrow("user").table}`);
}

export async function scalarRowAlias() {
  const table = getRowOrThrow("user").table;
  await query(`SELECT * FROM ${table}`);
}

export async function fieldRowAlias() {
  const selected = getRowOrThrow("user");
  const table = selected.table;
  await query(`SELECT * FROM ${table}`);
}
