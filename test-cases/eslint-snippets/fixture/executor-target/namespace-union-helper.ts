import * as db from "@example/db"; import { type QueryExecutor, read as namedRead } from "@example/db";
import * as other from "@example/other";
import * as sub from "@example/db/sub";
import type * as typeOnly from "@example/db";

export function union(query: db.QueryExecutor | typeof db.read) { query("SELECT 1"); }
export function parenthesized(query: ((db.QueryExecutor) | (typeof db.read))) { query("SELECT 1"); }
export function type(query: db.QueryExecutor) { query("SELECT 1"); }
export function value(query: typeof db.read) { query("SELECT 1"); }
export function mixed(query: db.QueryExecutor | ((statement: string) => unknown)) { query("SELECT 1"); }
export function wrongType(query: other.QueryExecutor) { query("SELECT 1"); }
export function wrongValue(query: typeof other.read) { query("SELECT 1"); }
export function mixedModule(query: db.QueryExecutor | typeof other.read) { query("SELECT 1"); }
export function subpath(query: sub.QueryExecutor) { query("SELECT 1"); }
export function wrongName(query: typeof db.write) { query("SELECT 1"); }
export function nested(query: db.nested.QueryExecutor) { query("SELECT 1"); }
export function typeOnlyValue(query: typeof typeOnly.read) { query("SELECT 1"); }

export function runtimeSubpath(query: typeof sub.read) { query("SELECT 1"); }
export function typeOnlyType(query: typeOnly.QueryExecutor) { query("SELECT 1"); }
export function importedParenthesizedType(query: ((QueryExecutor) | (typeof namedRead))) { query("SELECT 1"); }
export function shadowedNamespace(db: unknown, query: typeof db.read) { query("SELECT 1"); }
