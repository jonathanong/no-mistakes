import sql from "sql-template-strings";
import { read } from "@example/db";
import { FRAG } from "./fragments.mts";
import { ORDERS, computed, mutableTable } from "./tables.mts";

// The hole is still unresolved in this file, so the export is not a constant.
export const WRAPPED = `SELECT id FROM ${ORDERS} WHERE account_id = $1`;
// An interpolation is not a literal initializer, even when the hole is a string.
export const MIXED = `SELECT ${"id"} FROM orders WHERE account_id = $1`;

const T = "orders";
// `as const` is a separate shape from a plain string initializer.
const TConst = "orders" as const;
const COLS = { owner: "orders" };
const COLUMN_LIST = "id, account_id";
// A no-substitution template is a literal base, same as a quoted string.
const BASE = `SELECT id FROM orders WHERE `;
const QUERY = `SELECT id FROM orders WHERE account_id = $1`;
const LOCAL_FRAG = sql` AND account_id = $1`;

export function localConst(accountId: string) {
  return read(`SELECT id FROM ${T} WHERE account_id = $1`, [accountId]);
}

export function asConst(accountId: string) {
  return read(`SELECT id FROM ${TConst} WHERE account_id = $1`, [accountId]);
}

export function memberTemplate(accountId: string) {
  return read(`SELECT id FROM ${COLS.owner} WHERE account_id = $1`, [accountId]);
}

export function memberAppend() {
  const query = sql`SELECT id FROM `;
  query.append(COLS.owner);
  query.append(` WHERE account_id = $1`);
  return read(query);
}

export function columnList() {
  const query = sql`SELECT `;
  query.append(COLUMN_LIST);
  query.append(` FROM orders WHERE account_id = $1`);
  return read(query);
}

export function templateBase(accountId: string) {
  return read(`${BASE}account_id = $1`, [accountId]);
}

export function templateQuery(accountId: string) {
  return read(QUERY, [accountId]);
}

export function importedConst(accountId: string) {
  return read(`SELECT id FROM ${ORDERS} WHERE account_id = $1`, [accountId]);
}

export function importedAppend() {
  const query = sql`SELECT id FROM `;
  query.append(ORDERS);
  query.append(` WHERE account_id = $1`);
  return read(query);
}

export function importedFragment() {
  const query = sql`SELECT id FROM orders WHERE true`;
  query.append(FRAG);
  return read(query);
}

// Judged SQL that omits account_id. This is not an unanalyzable finding.
export function templateMissingColumn() {
  const missing = `SELECT id FROM orders WHERE true`;
  return read(missing);
}

// The imported constant is recovered, so the directive suppresses the column finding.
export function suppressed() {
  // no-mistakes-disable-next-line postgres-required-predicates
  return read(`SELECT id FROM ${ORDERS} WHERE true`);
}

let loose = "orders";
export function letBinding(accountId: string) {
  return read(`SELECT id FROM ${loose} WHERE account_id = $1`, [accountId]);
}

export function reassigned(accountId: string) {
  let rebound = "orders";
  rebound = "users";
  return read(`SELECT id FROM ${rebound} WHERE account_id = $1`, [accountId]);
}

function nameOfTable(): string {
  return "orders";
}
export function computedInit(accountId: string) {
  const name = nameOfTable();
  return read(`SELECT id FROM ${name} WHERE account_id = $1`, [accountId]);
}

export function mutatedMember(accountId: string) {
  const mutable = { owner: "orders" };
  mutable.owner = "users";
  return read(`SELECT id FROM ${mutable.owner} WHERE account_id = $1`, [accountId]);
}

// An alias is not the const binding. #2161 stays fail-closed.
export function alias(accountId: string) {
  const aliasOfT = T;
  return read(`SELECT id FROM ${aliasOfT} WHERE account_id = $1`, [accountId]);
}

// A container element is not a member of a constant object literal.
export function container(accountId: string) {
  const list = ["orders"];
  return read(`SELECT id FROM ${list[0]} WHERE account_id = $1`, [accountId]);
}

// A template with an interpolation is not a literal, even when the hole is a number.
export function interpolatedTemplate(accountId: string) {
  const dynamicTemplate = `SELECT ${1} FROM orders`;
  return read(`${dynamicTemplate} WHERE account_id = $1`, [accountId]);
}

// A tagged fragment stringifies inside an untagged template. Only .append copies it.
export function localFragmentTemplate() {
  return read(`SELECT id FROM orders WHERE true ${LOCAL_FRAG}`);
}

export function importedFragmentTemplate() {
  return read(`SELECT id FROM orders WHERE true ${FRAG}`);
}

export function importedLet(accountId: string) {
  return read(`SELECT id FROM ${mutableTable} WHERE account_id = $1`, [accountId]);
}

export function importedComputed(accountId: string) {
  return read(`SELECT id FROM ${computed} WHERE account_id = $1`, [accountId]);
}

// A parameter shadow is not the imported constant.
export function shadowedImport(accountId: string, ORDERS: string) {
  return read(`SELECT id FROM ${ORDERS} WHERE account_id = $1`, [accountId]);
}

// The sql tag import is not a string constant.
export function sqlTagPiece() {
  return read(`SELECT ${sql} id FROM orders WHERE account_id = $1`);
}
