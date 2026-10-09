// A type-only name must not replace the runtime String.raw built-in.
import type { String } from "./types.mjs";
import { write } from "@app/db";
import { sharedStatement, mutateSharedStatement } from "./shared-state.mjs";
import sql, { type SQLStatement } from "sql-template-strings";
import { annotatedOrdersSql, unannotatedOrdersSql } from "./sql-builders.mjs";
import defaultFunctionSql from "./default-function.mjs";
import defaultArrowSql from "./default-arrow.mjs";
import { importedOrdersSql, neverThere } from "./barrel.mjs";
import { ambiguousSql } from "./ambiguous-barrel.mjs";
import { customQuery, customQuery as assignedCustomQuery } from "./tags.mjs";

export async function letImportedHelperResult() {
  let result = annotatedOrdersSql();
  return write(result); // known:let-imported
}

export async function varImportedHelperResult() {
  var result = annotatedOrdersSql();
  return write(result); // known:var-imported
}

export async function letInlineImportedHelperResult() {
  let result = write(annotatedOrdersSql());
  return result; // known:let-inline-imported
}

export async function varInlineImportedHelperResult() {
  var result = write(annotatedOrdersSql());
  return result; // known:var-inline-imported
}

export async function importedUnannotatedHelperResult() {
  let result = unannotatedOrdersSql();
  return write(result); // finding:unannotated-import-ignore
}

export async function defaultFunctionHelperResult() {
  return write(defaultFunctionSql()); // known:default-function
}

export async function defaultArrowHelperResult() {
  return write(defaultArrowSql()); // known:default-arrow
}

export async function starBarrelHelperResult() {
  return write(importedOrdersSql()); // known:star-barrel
}

export async function cyclicStarExportIsOpaque() {
  return write(neverThere()); // unanalyzable:star-cycle
}

export async function ambiguousStarExportIsOpaque() {
  return write(ambiguousSql()); // unanalyzable:star-ambiguity
}

export async function configuredTransactionQueries(
  tx: { query(sql: string): Promise<unknown> },
  unrelated: { query(sql: string): Promise<unknown> },
) {
  tx.query("BEGIN"); tx.query("SELECT 1"); unrelated.query("/* unrelated */ SELECT 1"); // finding:transaction-same-line
}

function nestedShadow(customQuery: (strings: TemplateStringsArray) => SQLStatement) {
  return write(customQuery`/* local shadow */ SELECT id FROM orders`); // unanalyzable:nested-shadow
}

function nestedDestructuredTagShadow({ sql }: { sql: typeof customQuery }) {
  return write(sql`/* destructured local sql */ SELECT id FROM orders`); // unanalyzable:nested-destructured-sql
}

function nestedRestTagShadow(value: { sql: typeof customQuery; other: unknown }) {
  const { other, ...sql } = value;
  void other;
  return write(sql`/* rest local sql */ SELECT id FROM orders`); // unanalyzable:nested-rest-sql
}

export async function importedCustomTagSurvivesNestedShadow() {
  nestedShadow((strings) => sql(strings[0]));
  return write(customQuery`/* trusted custom */ SELECT id FROM orders`); // known:custom-import
}

assignedCustomQuery = (strings: TemplateStringsArray) => sql(strings[0]);

export async function assignedCustomImportIsOpaque() {
  return write(assignedCustomQuery`/* looks named */ SELECT id FROM orders`); // unanalyzable:assigned-custom-import
}

function strictAnnotatedHelper(): SQLStatement {
  "use strict";
  const statement = sql`/* strict helper */ SELECT id FROM orders`;
  return statement;
}

export async function strictHelperResult() {
  return write(strictAnnotatedHelper()); // known:strict-helper
}

function recognizedExecutorThenReturn(): SQLStatement {
  write(sql`/* inner executor */ SELECT 1`);
  return sql`/* helper after executor */ SELECT 2`;
}

export async function recognizedExecutorSideEffectThenReturn() {
  return write(recognizedExecutorThenReturn()); // known:executor-side-effect
}

export async function stringRawInterpolatedValue(id: string) {
  return write(String.raw`/* raw static */ SELECT ${id} FROM orders`); // known:string-raw-static
}

export async function stringRawDynamicFirst(prefix: string, id: string) {
  return write(String.raw`${prefix} /* too late */ SELECT ${id} FROM orders`); // unanalyzable:string-raw-dynamic-first
}

export async function liveCapturedValue() {
  const run = () => write(statement);
  const statement = sql`/* captured after declaration */ SELECT 1`;
  return run(); // known:live-capture
}

export async function earlyLiveCaptureIsOpaque() {
  const run = () => write(statement); // unanalyzable:early-live-capture
  run();
  const statement = sql`/* declared too late */ SELECT 1`;
}

export async function nestedCallerDoesNotReplaceCapturedValue() {
  const run = () => write(statement);
  const statement = sql`/* outer capture */ SELECT 1`;
  const nested = (statement: SQLStatement) => run();
  return nested(sql`SELECT 2`); // known:nested-shadowed-caller
}

export async function hoistedHelperDeclaredLater() {
  return write(hoistedAnnotatedSql()); // known:hoisted-helper
}

function hoistedAnnotatedSql(): SQLStatement {
  return sql`/* hoisted helper */ SELECT id FROM orders`;
}

export function computedTemplateQuery(client: { query(statement: SQLStatement): unknown }) {
  return client[`query`](annotatedOrdersSql()); // known:computed-template-query
}

async function asynchronousAnnotatedSql() {
  return annotatedOrdersSql();
}
async function asynchronousUnannotatedSql() {
  return unannotatedOrdersSql();
}
export async function awaitedHelperResults() {
  write(await asynchronousAnnotatedSql()); // known:await-annotated
  write(await asynchronousUnannotatedSql()); // finding:await-unannotated
  write(asynchronousAnnotatedSql()); // unanalyzable:unawaited-promise
}
export function initializerMutation() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  const ignored = unknownMutation(statement);
  write(statement); // unanalyzable:initializer-mutation
}
export function aliasArrayMutation() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  const alias = statement;
  const ignored = unknownMutation([alias]);
  write(statement); // unanalyzable:alias-array-mutation
}
export function primitiveStringIsNotMutable() {
  const statement = "/* immutable */ SELECT 1";
  const ignored = unknownMutation(statement);
  write(statement); // known:primitive-string
}
export function arbitraryTagMutation() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  arbitraryTag`${statement}`;
  write(statement); // unanalyzable:arbitrary-tag-mutation
}
export function erasedLocalDeclarations() {
  type Local = { id: number };
  interface Other {
    id: number;
  }
  write(annotatedOrdersSql()); // known:erased-local
  write(unannotatedOrdersSql()); // finding:erased-local-unannotated
}
export function templateSubstitutionExecutors() {
  const annotated = sql`SELECT ${write(annotatedOrdersSql())}`; // known:template-inner-annotated
  const missing = sql`SELECT ${write(unannotatedOrdersSql())}`; // finding:template-inner-unannotated
}
export function orderedNestedEffects() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  const results = [unknownMutation(statement), write(statement)]; // unanalyzable:ordered-effects
}
export function capturedBuilderMutation() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  function mutate() {
    statement.text = "SELECT 2";
  }
  const ignored = mutate();
  write(statement); // unanalyzable:captured-builder-mutation
}

export function appendMutatesAliases() {
  const statement = sql``;
  const alias = statement;
  alias.append("SELECT 1");
  write(statement); // finding:append-alias
}
export function appendTailCanInvalidateItsBase() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  write(statement.append(unknownMutation(statement))); // unanalyzable:append-tail-mutation
}

export function laterExecutorArgumentMutation() {
  const statement = sql`/* prior annotation */ SELECT 1`;
  write(statement, unknownMutation(statement)); // unanalyzable:later-argument-mutation
}
export async function promisedBuilderMutation() {
  const statement = asynchronousAnnotatedSql();
  const ignored = unknownMutation(statement);
  write(await statement); // unanalyzable:promised-builder-mutation
}

function unsupportedRawValue() {
  if (flag) return 'SELECT 1';
}
export function unsupportedRawInterpolation() {
  write(String.raw`${unsupportedRawValue()}`); // finding:unsupported-raw-interpolation
}

export function appendValueMutatesReceiver() {
  const statement = sql``;
  const alias = statement;
  const returned = alias.append('SELECT 1');
  write(statement); // finding:append-value-receiver
}
export function incompleteTransactions(tail: string) {
  write(sql``.append('BE').append(tail)); // unanalyzable:partial-begin
  write(sql``.append('COM').append(tail)); // unanalyzable:partial-commit
  write(sql``.append('ROLL').append(tail)); // unanalyzable:partial-rollback
  write(sql``.append('bEg').append(tail)); // unanalyzable:partial-case
  write(sql``.append('BEx').append(tail)); // finding:non-transaction-prefix
}

export function opaqueVarBuilderMutation() {
  var statement = sql`/* prior annotation */ SELECT 1`;
  const ignored = unknownMutation(statement);
  write(statement); // unanalyzable:opaque-var-builder-mutation
}
export function importedModuleCaptureMutation() {
  write(sharedStatement()); // known:initial-module-capture
  const ignored = mutateSharedStatement();
  write(sharedStatement()); // unanalyzable:mutated-module-capture
}
