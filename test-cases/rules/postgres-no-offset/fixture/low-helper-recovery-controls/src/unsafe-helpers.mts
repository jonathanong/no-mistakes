import SQL from 'sql-template-strings';
import type { QueryExecutor } from '@example/executor-types';

declare const opaque: (value: unknown) => unknown;

export function opaqueExecutor(query: QueryExecutor, sql: string) {
  opaque(query);
  return sql;
}

export function unsupportedGuard(enabled: boolean) {
  const statement = SQL` WHERE 1 = 1`;
  while (enabled) statement.append(SQL` OFFSET 1`);
  return statement;
}

export function rawFragment() {
  return SQL.raw(' OFFSET 1');
}

export function builderAlias() {
  const original = SQL` WHERE 1 = 1`;
  const alias = original;
  return alias;
}

export function builderMutation(value: string) {
  const statement = SQL` WHERE 1 = 1`;
  statement.append(value);
  return statement;
}

export function recursiveExecutor(query: QueryExecutor): unknown {
  return recursiveExecutor(query);
}

export function depth0(): string { return depth1(); }
function depth1(): string { return depth2(); }
function depth2(): string { return depth3(); }
function depth3(): string { return depth4(); }
function depth4(): string { return depth5(); }
function depth5(): string { return depth6(); }
function depth6(): string { return depth7(); }
function depth7(): string { return depth8(); }
function depth8(): string { return depth9(); }
function depth9(): string { return 'SELECT 1'; }

export function fiveVariants(flag: number) {
  return flag === 0 ? SQL` OFFSET 1`
    : flag === 1 ? SQL``
    : flag === 2 ? SQL``
    : flag === 3 ? SQL``
    : flag === 4 ? SQL``
    : SQL``;
}

export function smallWorkModel() {
  const statement = SQL` WHERE 1 = 1`;
  statement.append(SQL` AND active = true`);
  return statement;
}
