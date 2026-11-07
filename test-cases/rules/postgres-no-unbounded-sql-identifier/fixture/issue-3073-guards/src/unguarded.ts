import { query } from "@example/db";
import { validateIdentifier as validate } from "./barrel";
import { assertIdentifier } from "./other";

declare function assertSecond(context: string, value: string): void;
declare function mutate(config: { table: string }): void;

export async function unrelatedProperty(config: { schema: string; table: string }) {
  validate(config.schema);
  await query(`SELECT * FROM ${config.table}`);
}

export async function wrongModule(table: string) {
  assertIdentifier(table);
  await query(`SELECT * FROM ${table}`);
}

export async function shadowedModule(table: string) {
  const validate = (value: string) => {};
  validate(table);
  await query(`SELECT * FROM ${table}`);
}

export async function wrongArgument(table: string) {
  assertSecond(table, "users");
  await query(`SELECT * FROM ${table}`);
}

export async function reassigned(config: { table: string }) {
  validate(config.table);
  config.table = process.env.TABLE;
  await query(`SELECT * FROM ${config.table}`);
}

export async function escaped(config: { table: string }) {
  validate(config.table);
  mutate(config);
  await query(`SELECT * FROM ${config.table}`);
}

export async function aliased(config: { table: string }) {
  const alias = config;
  validate(config.table);
  alias.table = process.env.TABLE;
  await query(`SELECT * FROM ${config.table}`);
}

export async function validatedTooLate(table: string) {
  const sql = `SELECT * FROM ${table}`;
  validate(table);
  await query(sql);
}

export async function branchOnly(table: string, enabled: boolean) {
  if (enabled) validate(table);
  await query(`SELECT * FROM ${table}`);
}

const OPEN = ["users", process.env.TABLE];
export async function openIncludes(table: string) {
  if (!OPEN.includes(table)) return;
  await query(`SELECT * FROM ${table}`);
}

const CLOSED = ["users", "posts"];
export async function nonTerminatingGuard(table: string) {
  if (!CLOSED.includes(table)) console.error("Unknown table");
  await query(`SELECT * FROM ${table}`);
}

export async function shadowedSubject(table: string) {
  validate(table);
  {
    const table = process.env.TABLE;
    await query(`SELECT * FROM ${table}`);
  }
}
