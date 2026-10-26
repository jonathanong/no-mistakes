import SQL, { type SQLStatement } from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
function identity(value: unknown) { return value; }
function memberIdentity(value: { statement: SQLStatement }) { return value.statement; }
function unsafeReadonly(execute: QueryExecutor, statement: SQLStatement) {
  const same = identity(statement);
  same.append(SQL` OFFSET 1`);
  return execute(statement);
}
function unsafeNested(execute: QueryExecutor, statement: SQLStatement) {
  const same = memberIdentity({ statement });
  same.append(SQL` OFFSET 1`);
  return execute(statement);
}
function readonlyQuery(execute: QueryExecutor, statement: SQLStatement) {
  return execute(identity(statement));
}
function readonlyFragment(execute: QueryExecutor, statement: SQLStatement) {
  return execute(SQL`SELECT 1`.append(identity(statement)));
}
function fresh(options: { enabled: boolean }) {
  return options.enabled ? SQL`SELECT 1 OFFSET 4` : SQL`SELECT 1`;
}
export function freshLocalBuilder() {
  const statement = fresh({ enabled: false });
  statement.append(SQL` WHERE 1 = 1`);
  read(statement); // FRESH_CLEAR: no incoming SQL can alias this helper result.
}
export function callerOwnedLocal() {
  const statement = SQL`SELECT 1`;
  const same = identity(statement);
  same.append(SQL` OFFSET 1`);
  read(statement); // LOCAL_UNKNOWN: caller-owned mutation remains unsupported.
}
export function aliasBoundaries() {
  unsafeReadonly(read, SQL`SELECT 1`); // READONLY_UNKNOWN: returned SQL may alias the parameter.
  unsafeNested(read, SQL`SELECT 1`); // NESTED_UNKNOWN: a record does not erase incoming SQL identity.
  readonlyQuery(read, SQL`SELECT 1`); // READONLY_CLEAR: reading an identity result does not mutate it.
  readonlyFragment(read, SQL` OFFSET 2`); // FRAGMENT_OFFSET: a readonly result can still be appended as RHS text.
}
