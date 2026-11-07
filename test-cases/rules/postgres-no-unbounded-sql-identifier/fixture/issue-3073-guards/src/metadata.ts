import { query } from "@example/db";

const TABLES = [
  { name: "user", schema: "public", table: "users" },
  { name: "post", schema: "archive", table: "posts" },
] as const;

export async function lookupOrThrow(name: string) {
  const selected = TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  await query(`SELECT * FROM ${selected.schema}.${selected.table}`);
}

export async function lookupOrReturn(name: string) {
  const selected = TABLES.find((entry) => name === entry.name);
  if (!selected) return;
  await query(`SELECT * FROM ${selected.table}`);
}

const OPEN_TABLES = [{ name: "user", table: process.env.TABLE }];
export async function openLookup(name: string) {
  const selected = OPEN_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  await query(`SELECT * FROM ${selected.table}`);
}

const MUTABLE_TABLES = [{ name: "user", table: "users" }];
export async function mutatedLookup(name: string) {
  const selected = MUTABLE_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  selected.table = process.env.TABLE;
  await query(`SELECT * FROM ${selected.table}`);
}

const EFFECT_TABLES = [{ name: "user", table: "users" }];
export async function effectLookup(name: string) {
  const selected = EFFECT_TABLES.find((entry) => (entry.table = process.env.TABLE, entry.name === name));
  if (!selected) throw new Error("Unknown table");
  await query(`SELECT * FROM ${selected.table}`);
}

const ESCAPED_TABLES = [{ name: "user", table: "users" }];
const escapedEntry = ESCAPED_TABLES[0];
escapedEntry.table = process.env.TABLE;
export async function aliasedLookup(name: string) {
  const selected = ESCAPED_TABLES.find((entry) => entry.name === name);
  if (!selected) throw new Error("Unknown table");
  await query(`SELECT * FROM ${selected.table}`);
}
