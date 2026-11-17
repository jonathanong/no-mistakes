import * as db from "@example/db"; import { type QueryExecutor, read as namedRead } from "@example/db"; import { read } from "@example/db";
import * as other from "@example/other";
import * as sub from "@example/db/sub";
import type * as typeOnly from "@example/db";

function union(query: db.QueryExecutor | typeof db.read) { query("SELECT 1"); }
function parenthesized(query: ((db.QueryExecutor) | (typeof db.read))) { query("SELECT 1"); }
function type(query: db.QueryExecutor) { query("SELECT 1"); }
function value(query: typeof db.read) { query("SELECT 1"); }
function mixed(query: db.QueryExecutor | ((statement: string) => unknown)) { query("SELECT 1"); }
function wrongType(query: other.QueryExecutor) { query("SELECT 1"); }
function wrongValue(query: typeof other.read) { query("SELECT 1"); }
function mixedModule(query: db.QueryExecutor | typeof other.read) { query("SELECT 1"); }
function subpath(query: sub.QueryExecutor) { query("SELECT 1"); }
function wrongName(query: typeof db.write) { query("SELECT 1"); }
function nested(query: db.nested.QueryExecutor) { query("SELECT 1"); }
function typeOnlyValue(query: typeof typeOnly.read) { query("SELECT 1"); }
function shadow(db: unknown) {
  function hidden(query: typeof db.read) { query("SELECT 1"); }
  hidden(read);
}

union(db.read);
parenthesized(db.read);
type(db.read);
value(db.read);
mixed(db.read);
wrongType(db.read);
wrongValue(db.read);
mixedModule(db.read);
subpath(db.read);
wrongName(db.read);
nested(db.read);
typeOnlyValue(db.read);
shadow(db);

function runtimeSubpath(query: typeof sub.read) { query("SELECT 1"); }
function typeOnlyType(query: typeOnly.QueryExecutor) { query("SELECT 1"); }
function importedParenthesizedType(query: ((QueryExecutor) | (typeof namedRead))) { query("SELECT 1"); }
runtimeSubpath(db.read);
typeOnlyType(db.read);
importedParenthesizedType(db.read);
