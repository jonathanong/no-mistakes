import SQL, { type SQLStatement } from 'sql-template-strings';
import { read } from '@example/db';
import type { QueryExecutor, QueryOptions } from '@example/executor-types';
import {
  forwardNamed,
  forwardBuilder,
  forwardOptions,
  freshClause,
  qualifier,
  fixedClause,
  callerFragment as appendCallerFragment,
  optionalOffset,
} from './helpers.mjs';

declare const dynamicSql: string;

export function namedForwarder(query: QueryExecutor) {
  forwardNamed(query, 'SELECT 1'); // EXPECT_CLEAR: typed executor and caller literal cross imported helper calls.
}

export function namedForwarderWithBinds(query: QueryExecutor, id: string) {
  forwardBuilder(query, SQL`SELECT ${id} AS id`); // EXPECT_CLEAR: caller bind SQL flows through typed helper parameters.
}

export function destructuredOptions(options: QueryOptions) {
  forwardOptions(options, 'SELECT 1'); // EXPECT_CLEAR: options.query is the configured executor member.
}

export function freshBuilder(enabled: boolean, id: string) {
  read(SQL`SELECT ${id} AS id`.append(freshClause({ enabled, value: id }))); // EXPECT_CLEAR: every SQL tag interpolation is a bind.
  read(SQL`SELECT ${id} AS id`.append(freshClause())); // EXPECT_CLEAR: omitted options use the helper's default.
}

function sameFileClause({ value = 'ready' }: { value?: string } = {}) {
  return SQL` AND value = ${value}`;
}
export function sameFileFreshBuilder(id: string) {
  read(SQL`SELECT ${id} AS id`.append(sameFileClause({ value: id }))); // EXPECT_CLEAR: same-file helper returns a fresh parameterized builder.
}

export function noArgumentAndQualifier() {
  read(SQL`SELECT 1 WHERE `.append(qualifier('item'))); // EXPECT_CLEAR: literal qualifier flows through a helper parameter.
  read(SQL`SELECT 1`.append(fixedClause())); // EXPECT_CLEAR: no argument helper returns a fresh builder.
}

export function callerFragment() {
  read(SQL`SELECT 1`.append(appendCallerFragment(' OFFSET 1'))); // EXPECT_OFFSET_CALLER: caller literal fragment reaches the appended SQL.
}

export function helperOrigin() {
  read(SQL`SELECT 1`.append(optionalOffset({ enabled: true }))); // EXPECT_OFFSET_HELPER: report the helper source line for appended OFFSET.
}

export function nestedStatementIsAValue(value: SQLStatement) {
  read(SQL`SELECT ${value} AS payload`); // EXPECT_CLEAR: nested SQLStatement is a bind operand, never inlined SQL text.
}

export function dynamicControl() {
  read(dynamicSql); // EXPECT_ADVISE_DYNAMIC: a dynamic SQL argument remains unknown.
}
