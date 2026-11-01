import { beginTransaction, read, write } from '@example/db';
import type { QueryExecutor, QueryOptions } from '@example/types';
import sql from 'sql-template-strings';
import { active, dynamicName } from './helpers.mjs';

declare const dynamicSql: string;
declare const table: string;
declare const columns: string;
declare const tempTable: string;
declare const limit: number;
declare const id: string;
declare const plan: { table: string };
declare const ids: string[];
declare const options: QueryOptions;
declare function other(value: QueryOptions): void;

type Local = QueryExecutor;
type Transaction = Awaited<ReturnType<typeof beginTransaction>>;

export function aliasLiteral(run: Local, value: string, relation: string) {
  run(`SELECT 1`);
  run(sql`SELECT ${value} AS id`);
  run(`SELECT * FROM ${relation}`);
}

export function transactionAlias(run: Transaction, value: string, relation: string) {
  run(`SELECT 1`);
  run(sql`SELECT ${value} AS id`);
  run(`SELECT * FROM ${relation}`);
}

export async function typeofTransaction(value: string, relation: string) {
  const transaction = await beginTransaction();
  function receiving(run: typeof transaction) {
    run(`SELECT 1`);
    run(sql`SELECT ${value} AS id`);
    run(`SELECT * FROM ${relation}`);
  }
  receiving(transaction);
}

function compare(query: QueryExecutor, left: number, right: number) {
  query('SELECT 1');
  return left < right;
}
function lengthOf(query: QueryExecutor, value: string) {
  query('SELECT 1');
  return value.length + 1;
}
function firstRow(query: QueryExecutor, rows: unknown[]) {
  query('SELECT 1');
  return rows[0];
}
function mapped(query: QueryExecutor, rows: { id: string }[]) {
  query('SELECT 1');
  return rows.map((row) => row.id);
}
function flagged(query: QueryExecutor, value: unknown) {
  query('SELECT 1');
  return Boolean(value);
}
function optionalRows(query: QueryExecutor) {
  return query('SELECT 1')?.rows;
}
function notEqual(query: QueryExecutor, value: unknown) {
  query('SELECT 1');
  return value !== 0;
}
function dynamicReturn(query: QueryExecutor, relation: string) {
  return query(`SELECT * FROM ${relation}`);
}

compare(read, 1, 2);
lengthOf(read, 'value');
firstRow(read, []);
mapped(read, []);
flagged(read, id);
optionalRows(read);
notEqual(read, 1);
dynamicReturn(read, table);

read('SELECT 1', { query: write });
read(`SELECT * FROM ${table}`, { query: write });
write('SELECT 1', { query: read });

function execute(query: QueryExecutor) {
  query('SELECT 1');
}
execute(options?.query ?? read);
function executeDynamic(query: QueryExecutor) {
  query(dynamicSql);
}
executeDynamic(options?.query ?? read);
(options?.query ?? read)('SELECT 1');
(options?.query ?? read)(`SELECT * FROM ${table}`);

const run = options.query ?? write;
run('SELECT 1');
run(`SELECT * FROM ${table}`);
run(sql`SELECT ${id} AS id`);

const alias = read;
alias('SELECT 1');
const guarded = options.query;
guarded('SELECT 1');

function unused(carrier: QueryOptions) {
  return carrier.limit;
}
unused({ query: read });
function forwards(carrier: QueryOptions) {
  other(carrier);
}
forwards({ query: read });
function usesQuery(carrier: QueryOptions) {
  return carrier.query ?? read;
}
usesQuery({ query: write });

function bound(query: QueryExecutor, count: number) {
  return query(sql`SELECT id FROM users LIMIT ${count + 1}`);
}
bound(read, limit);

function looped(query: QueryExecutor, values: string[]) {
  const statement = sql`SELECT id FROM users WHERE false`;
  for (const value of values) {
    statement.append(sql` OR id = ${value}`);
  }
  return query(statement);
}
looped(read, ids);

function conditional(query: QueryExecutor, filter: { status?: string }) {
  const { status = 'open' } = filter;
  return query(sql`SELECT id FROM users WHERE status = ${status}`);
}
conditional(read, {});

function appended(query: QueryExecutor) {
  const statement = sql`SELECT id FROM users WHERE true`;
  statement.append(active());
  return query(statement);
}
appended(read);

function badAppend(query: QueryExecutor, relation: string) {
  const statement = sql`SELECT id FROM users`;
  statement.append(` UNION SELECT * FROM ${relation}`);
  return query(statement);
}
badAppend(read, tempTable);

function poisoned(query: QueryExecutor, relation: string) {
  return query(dynamicName(relation));
}
poisoned(read, table);

read(`SELECT ${columns} FROM users`);
read(`SELECT * FROM ${plan.table}`);
read(`SELECT * FROM ${tempTable}`);
read('SELECT id FROM users WHERE lower(id::text) = $1');
