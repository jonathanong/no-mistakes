import { write } from './db.mjs';
import sql from 'sql-template-strings';
import { missing } from './missing.mjs';
import { absent } from './exports.mjs';
import { invalid } from './invalid.mjs';
import { loop } from './cycle-a.mjs';
import { lost } from './reexport-missing.mjs';
import { fragment as configuredTag } from '@custom/sql/fragment';

function strings() { return 'SELECT ' + `1`; }
function mutated() {
  const statement = sql``;
  statement.append('SELECT 1');
  ;
  return statement;
}
const anonymous = function () { return 'SELECT 1'; };
const named = function self() { return self(); };
const arrowBlock = () => { return sql`SELECT 1`; };
function nothing() { return; }
function empty() {}
declare function declared(): unknown;
function params(statement: unknown) { return statement; }
export async function ordinary() {
  write(strings()); // finding:binary
  write(mutated()); // finding:append-mutation
  write(anonymous()); // finding:function-expression
  write(arrowBlock()); // finding:block-arrow
  write(named()); // unanalyzable:named-expression-self-binding
  write(nothing()); // unanalyzable:return-no-argument
  write(empty()); // unanalyzable:empty-body
  write(declared()); // unanalyzable:declared-function
  write(missing()); // unanalyzable:missing-module
  write(absent()); // unanalyzable:missing-export
  write(invalid()); // unanalyzable:parse-failed-helper
  write(loop()); // unanalyzable:reexport-cycle
  write(lost()); // unanalyzable:reexport-missing-module
  write(params()); // unanalyzable:missing-argument
  write(params(...[])); // unanalyzable:spread-argument
  write(params?.('SELECT 1')); // unanalyzable:optional-call
  write(sql``.append()); // unanalyzable:append-missing-argument
  write(sql``.append?.('SELECT 1')); // unanalyzable:optional-append
  write((0, params)('SELECT 1')); // unanalyzable:unknown-call-target
  write(unknown().append(sql`/* later */ SELECT 1`)); // unanalyzable:unknown-base
  write(String.raw`/* raw */ SELECT 1`); // known:raw-template
  write(configuredTag`/* trusted subpath */ SELECT 1`); // known:trusted-subpath
  write(String.raw`SELECT ${'id'}`); // unanalyzable:interpolated-raw-template
  write(sql``.append('/*').append(unknown())); // unanalyzable:partial-comment
  write(sql``.append('/x').append(unknown())); // finding:non-comment-slash
  write(sql``.append('-- line').append(unknown())); // finding:line-comment
  write(sql``.append('/* */ SELECT ').append(unknown())); // finding:empty-comment
  write(sql``.append('BEGIN').append(unknown())); // unanalyzable:partial-transaction-boundary
  write(sql``.append(' ').append(unknown())); // unanalyzable:whitespace-prefix
  write(sql``.append(sql`/* stable */ `).append(unknown())); // known:stable-prefix
  await write('SELECT 1'); // finding:await-expression
  database.query(sql`SELECT 1`); // finding:member-executor
  getDatabase().query(sql`SELECT 1`); // finding:complex-member-executor
  database['query'](strings()); // finding:computed-member-executor
}

const globalStatement = sql``;
function appendsGlobal() { globalStatement.append(unknown()); return globalStatement; }
export function globalMutation() { write(appendsGlobal()); } // unanalyzable:global-append
export class Opaque {}
globalStatement.append(unknown());
const { destructured } = external;
export let mutable = external;

function forward(statement: unknown, run: (value: unknown) => unknown) { return run(statement); }
const escaping = function self(statement: unknown) { return write(statement); }; // unanalyzable:escaped-callback
export function knownAndOpaqueCallbackCallsites() {
  forward(sql`/* known */ SELECT 1`, escaping);
  external(escaping);
  external(({ statement }: { statement: unknown }) => write(statement)); // unanalyzable:unsupported-callback-argument
}
function unsupportedForward(statement: unknown, run: (value: unknown) => unknown) {
  if (unknown()) return run(statement);
}
const unsupportedEscape = (statement: unknown) => write(statement); // unanalyzable:unsupported-wrapper-callback
export function opaqueForwardingCallsite() {
  unsupportedForward(sql`/* known */ SELECT 1`, unsupportedEscape);
}

function unsupportedSql() { if (unknown()) return sql`SELECT 1`; }
export function unsupportedComposition() {
  write(unsupportedSql().append(sql`/* later */ SELECT 1`)); // unanalyzable:unsupported-base
  write(sql``.append(unsupportedSql())); // unanalyzable:unsupported-leading-tail
  let dynamicWrapper = external;
  dynamicWrapper(sql`/* named */ SELECT 1`, (statement: unknown) => write(statement)); // unanalyzable:unsupported-target-callback
}
function reassignedGlobalHelper() { return sql`/* initially safe */ SELECT 1`; }
reassignedGlobalHelper = external;
export function globalHelperRevoked() { write(reassignedGlobalHelper()); } // unanalyzable:reassigned-global-helper
params?.();
