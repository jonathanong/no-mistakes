import SQL, { type SQLStatement } from 'sql-template-strings';
import { read } from '@example/db';
import type { QueryExecutor, QueryOptions } from '@example/executor-types';

export function executeNamed(query: QueryExecutor, sql: string) {
  return query(sql);
}

export function forwardNamed(query: QueryExecutor, sql: string) {
  return executeNamed(query, sql);
}

export function executeBuilder(query: QueryExecutor, statement: SQLStatement) {
  return query(statement);
}

export function forwardBuilder(query: QueryExecutor, statement: SQLStatement) {
  return executeBuilder(query, statement);
}

export function forwardOptions(options: QueryOptions, sql: string) {
  const { query = read } = options;
  return query(sql);
}

export function freshClause(
  { enabled = false, value }: { enabled?: boolean; value?: string } = {},
): SQLStatement {
  const statement = SQL` WHERE 1 = 1`;
  if (enabled) statement.append(SQL` AND value = ${value}`);
  return statement;
}

export function qualifier(alias: string) {
  return `${alias}.deleted_at IS NULL`;
}

export function fixedClause() {
  return SQL` AND active = true`;
}

export function callerFragment(fragment: string) {
  return fragment;
}

export function optionalOffset({ enabled = false }: { enabled?: boolean } = {}) {
  return enabled ? SQL` OFFSET 1` : SQL``; // EXPECT_OFFSET_HELPER_ORIGIN
}
