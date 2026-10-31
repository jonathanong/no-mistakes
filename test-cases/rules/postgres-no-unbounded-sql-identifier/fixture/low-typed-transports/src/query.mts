import sql from 'sql-template-strings';
import { beginTransaction, read } from '@example/db';
import type { QueryExecutor, TransactionQuery } from '@example/db';
import { typed, untyped, nonExecutor, wrongPosition, otherType, mixed } from './receivers.mjs';
import { excluded } from './excluded.mjs';

declare const value: string;
export async function execute(run: QueryExecutor) {
  run(sql`SELECT * FROM accounts WHERE id = ${value}::uuid`); // CLEAR_TAG
  run(sql`SELECT * FROM accounts WHERE id::text = ${value}`); // UUID_TAG
  typed(run); // CLEAR_TYPED
  localTyped(run); // CLEAR_TYPED
  untyped(run); // UNKNOWN_UNTYPED
  nonExecutor(run); // UNKNOWN_NON_EXECUTOR
  mixed(run, run); // UNKNOWN_MIXED
  wrongPosition(run, value); // UNKNOWN_POSITION
  otherType(run); // UNKNOWN_TYPE
  excluded(run); // UNKNOWN_EXCLUDED
  typed({ run }); // UNKNOWN_CONTAINER
  typed(...[run]); // UNKNOWN_SPREAD
  replaced(run); // UNKNOWN_REASSIGNED
  { const typed = opaque; typed(run); } // UNKNOWN_SHADOWED
  const transaction = await beginTransaction();
  const annotated: TransactionQuery = await beginTransaction();
  typed(annotated); // CLEAR_TYPED
  typed(transaction); // CLEAR_TYPED
  transaction(sql`SELECT * FROM accounts WHERE id = ${value}::uuid`); // CLEAR_TAG
  arrowTyped(transaction); // CLEAR_TYPED
  expressionTyped(transaction); // CLEAR_TYPED
  untyped(transaction); // UNKNOWN_UNTYPED
  run(value); // UNKNOWN_SQL
}
function localTyped(run: QueryExecutor) {
  for (const item of values) opaque(item);
  run('SELECT 1');
}
declare const values: string[];
declare function opaque(value: unknown): void;
function replaced(run: QueryExecutor) { run('SELECT 1'); }
replaced = opaque;
const arrowTyped = (run: QueryExecutor) => { opaque(value); run('SELECT 1'); };
export function transactionTyped(run: TransactionQuery) {
  typed(run); // CLEAR_TYPED
}
read(`SELECT * FROM ${process.env.TABLE}`); // IDENTIFIER
const expressionTyped = function(run: QueryExecutor) { run('SELECT 1'); };

export function callback(transaction: TransactionQuery, run: (query: QueryExecutor) => void) {
  run(transaction); // CLEAR_TYPED
}
export function callbackUnknown(transaction: TransactionQuery, run: (query) => void) {
  run(transaction); // UNKNOWN_CALLBACK
}
export function callbackWrong(transaction: TransactionQuery, run: (first: unknown, query: QueryExecutor) => void) {
  run(transaction, value); // UNKNOWN_CALLBACK_POSITION
}
export function callbackSpread(transaction: TransactionQuery, run: (query: QueryExecutor) => void) {
  run(...[transaction]); // UNKNOWN_CALLBACK_SPREAD
}
