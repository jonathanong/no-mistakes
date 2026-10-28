import { query } from '@example/db';
import SQL from 'sql-template-strings';
import { safeSql, dropSql, sleepSql, brokenSql, uncertainSql, deep1, mutableSql, abrupt } from '../helpers/sql';
import { externalSql } from 'opaque-external-sql';

query(safeSql()); // recovered safe helper
query(dropSql()); // recovered DROP helper
query(sleepSql()); // recovered sleep helper

declare const flag: boolean;
query(uncertainSql(flag)); // uncertain helper branch
query(brokenSql()); // invalid helper SQL
query(mutableSql); // exported mutable SQL
query(externalSql()); // external wrapper
query(deep1()); // bounded depth residual

declare function escape(value: unknown): void;
function escapedSql() {
  const statement = SQL`SELECT 1`;
  escape(statement);
  return statement;
}
query(escapedSql()); // escaped helper

declare const mode: string;
query(abrupt(mode)); // throw argument has SQL effect
