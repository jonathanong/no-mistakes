import sql from "sql-template-strings";
import { read } from "@example/db";
import * as ns from "./constants.mts";
import * as renamed from "./barrel.mts";
import type * as types from "./constants.mts";
import * as absent from "./missing.mts";
import { object as named } from "./constants.mts";

export function direct() { return read(`SELECT id FROM ${ns.TABLE} WHERE account_id = $1`); }
export function template() { return read(`${ns.QUERY}`); }
export function defaultMember() { return read(`SELECT id FROM ${ns.default} WHERE account_id = $1`); }
export function alias() { return read(`SELECT id FROM ${renamed.RENAMED} WHERE account_id = $1`); }
export function star() { return read(`SELECT id FROM ${renamed.TABLE} WHERE account_id = $1`); }
export function reexportDefault() { return read(`SELECT id FROM ${renamed.DEFAULT_TABLE} WHERE account_id = $1`); }
export function append() { const q = sql`SELECT id FROM `; q.append(ns.TABLE); q.append(` WHERE account_id = $1`); return read(q); }
export function fragment() { const q = sql`SELECT id FROM orders WHERE true`; q.append(ns.FRAGMENT); return read(q); }
export function missingPredicate() { return read(`${ns.MISSING}`); }

// Similar spellings must remain unknown unless the export is a static constant.
export function shadow(ns: unknown) { return read(`SELECT id FROM ${ns.TABLE} WHERE account_id = $1`); }
export function typeOnly() { return read(`SELECT id FROM ${types.TABLE} WHERE account_id = $1`); }
export function computed() { return read(`SELECT id FROM ${ns["TABLE"]} WHERE account_id = $1`); }
export function method() { return read(`SELECT id FROM ${ns.tableName()} WHERE account_id = $1`); }
export function mutable() { return read(`SELECT id FROM ${ns.mutable} WHERE account_id = $1`); }
export function dynamic() { return read(`SELECT id FROM ${ns.dynamic} WHERE account_id = $1`); }
export function unresolved() { return read(`SELECT id FROM ${absent.TABLE} WHERE account_id = $1`); }
export function optional() { return read(`SELECT id FROM ${ns?.TABLE} WHERE account_id = $1`); }
export function nested() { return read(`SELECT id FROM ${ns.object.TABLE} WHERE account_id = $1`); }
export function namedObject() { return read(`SELECT id FROM ${named.TABLE} WHERE account_id = $1`); }
// Untagged interpolation stringifies fragments; append above copies their SQL.
export function fragmentTemplate() { return read(`SELECT id FROM orders WHERE true ${ns.FRAGMENT}`); }
// Existing local object-member splicing retains precedence for a shadowed namespace.
export function localObject() { const ns = { TABLE: "orders" }; return read(`SELECT id FROM ${ns.TABLE} WHERE account_id = $1`); }
export function appendTemplate() { const q = sql`SELECT id FROM `; q.append(`${ns.TABLE}`); q.append(` WHERE account_id = $1`); return read(q); }
export function appendFragmentTemplate() { const q = sql`SELECT id FROM orders WHERE true`; q.append(`${ns.FRAGMENT}`); return read(q); }
