import SQL from 'sql-template-strings';
import { read } from '@example/db';
import { fragment } from './helpers.mjs';

export function accepted(id: string, enabled: boolean) {
  const options = { enabled, value: id }; // NON_SQL_OPTIONS
  const label = id; // NON_SQL_LOCAL
  read(SQL`SELECT ${id} AS id`); // CLEAR_SINGLE_BIND
  read(SQL`SELECT
    ${id} AS id
  `); // CLEAR_MULTILINE_BIND
  read(SQL`SELECT ${id} AS id`.append(SQL` WHERE ${enabled} OR true`)); // CLEAR_TEMPLATE_APPEND
  const statement = SQL`SELECT ${id} AS id`; // NON_SQL_BUILDER
  statement.append(fragment(options)); // NON_SQL_APPEND
  read(statement); // CLEAR_BUILDER_APPEND
  return label; // NON_SQL_RETURN
}

function untyped(execute: any, statement: any) { return execute(statement); }
declare const opaque: (value: unknown) => unknown;
export function uncertain(statement: string) {
  read(statement); // UNKNOWN_DYNAMIC
  untyped(read, statement); // UNKNOWN_UNTYPED
  opaque(read); // UNKNOWN_OPAQUE
}
