import { query } from "@example/db";
import { validateIdentifier as validate, validateRemote } from "./barrel";
import * as validators from "./barrel";

declare function assertSecond(context: string, value: string): void;
declare function assertLegacy(value: string): void;

export async function envAlias() {
  const schema = process.env.SCHEMA;
  validate(schema);
  await query(`SELECT * FROM ${schema}.users`);
}

export async function property(config: { schema: string; table: string }) {
  validate(config.schema);
  validateRemote(config.table);
  await query(`SELECT * FROM ${config.schema}.${config.table}`);
}

export async function namespace(table: string) {
  validators.validateIdentifier(table);
  await query(`SELECT * FROM ${table}`);
}

export async function argument(table: string) {
  assertSecond("table", table);
  await query(`SELECT * FROM ${table}`);
}

export async function legacy(config: { table: string }) {
  assertLegacy(config.table);
  await query(`SELECT * FROM ${config.table}`);
}

const ALLOWED_SCHEMAS = ["public", "archive"] as const;
export async function includesReturn(config: { schema: string }) {
  if (!ALLOWED_SCHEMAS.includes(config.schema)) return;
  await query(`SELECT * FROM ${config.schema}.users`);
}

const ALLOWED_TABLES = new Set(["users", "posts"]);
export async function setThrow(table: string) {
  if (!ALLOWED_TABLES.has(table)) throw new Error("Unknown table");
  await query(`SELECT * FROM ${table}`);
}

export async function localBinding() {
  const config = { table: process.env.TABLE };
  validate(config["table"]);
  await query(`SELECT * FROM ${config.table}`);
}

const STATIC_CONFIG = { schema: "public", table: "users" } as const;
const configuredTable = STATIC_CONFIG.table;
const configuredSql = `SELECT * FROM ${STATIC_CONFIG.schema}.${configuredTable}`;
export async function staticConfiguration() {
  await query(configuredSql);
}
