import SQL from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
import type { QueryExecutor as TypedExecutor } from '@example/executor-types';
import { ambiguousFragment } from './ambiguous.mjs';
import { missingFragment } from './missing.mjs';
import {
  opaqueExecutor,
  unsupportedGuard,
  rawFragment,
  builderAlias,
  builderMutation,
  recursiveExecutor,
  depth0,
  fiveVariants,
  smallWorkModel,
} from './unsafe-helpers.mjs';

declare const dynamicSql: string;
declare const flag: boolean;
declare const choice: number;

export function opaqueEscape(query: TypedExecutor, sql: string) {
  opaqueExecutor(query, sql); // EXPECT_ADVISE_ESCAPE: a typed executor passed through an opaque helper stays visible.
}

export function unsupportedInputs() {
  read(SQL`SELECT 1`.append(unsupportedGuard(flag))); // EXPECT_ADVISE_GUARD: unsupported control flow cannot prove a complete fragment.
  read(SQL`SELECT 1`.append(rawFragment())); // EXPECT_ADVISE_RAW: SQL.raw is outside the closed fragment model.
  read(dynamicSql); // EXPECT_ADVISE_DYNAMIC
}

export function unresolvedImports() {
  read(SQL`SELECT 1`.append(ambiguousFragment())); // EXPECT_ADVISE_AMBIGUOUS: duplicate barrel exports do not select one helper.
  read(SQL`SELECT 1`.append(missingFragment())); // EXPECT_ADVISE_UNRESOLVED: an unresolved module stays opaque.
}

export function executorRebinding(query: QueryExecutor) {
  let current: QueryExecutor = query;
  current = opaqueExecutor as unknown as QueryExecutor; // EXPECT_ADVISE_REBIND
  current('SELECT 1'); // EXPECT_ADVISE_REBIND_CALL
}

export function shadowDoesNotInheritExecutor(query: QueryExecutor) {
  {
    const query: (sql: string) => unknown = dynamicSql as unknown as (sql: string) => unknown;
    query('SELECT 1 OFFSET 2'); // EXPECT_NO_CROSS_BINDING: local shadow is not the typed parameter.
  }
  read('SELECT 1 OFFSET 3'); // EXPECT_FINDING: unrelated configured call remains independently analyzable.
}

export function mutatedBuilders(value: string) {
  read(SQL`SELECT 1`.append(builderAlias())); // EXPECT_ADVISE_ALIAS: returning an aliased builder is not a fresh-builder proof.
  read(SQL`SELECT 1`.append(builderMutation(value))); // EXPECT_ADVISE_MUTATION: dynamic builder mutation leaves an opaque remainder.
  read(SQL`SELECT 1`.append(fiveVariants(choice))); // EXPECT_ADVISE_VARIANTS_TEST_OVERRIDE: five alternatives exceed the private test cap.
  read(SQL`SELECT 1`.append(smallWorkModel())); // EXPECT_ADVISE_WORK_TEST_OVERRIDE: a small model exceeds the private test work cap.
  read(recursiveExecutor(read)); // EXPECT_ADVISE_RECURSION: recursive helper cycles fail closed.
  read(depth0()); // EXPECT_ADVISE_DEPTH: helper call depth is bounded.
}

export function unsafeTemplateBindIsStillData(value: TypedExecutor) {
  read(SQL`SELECT ${SQL` OFFSET 1`} AS payload`); // EXPECT_CLEAR_NESTED_BIND: nested SQLStatement is parameter data.
  read(SQL`SELECT ${value} AS payload`); // EXPECT_CLEAR_BIND_DATA: bound values are data even when their source type is callable.
}
