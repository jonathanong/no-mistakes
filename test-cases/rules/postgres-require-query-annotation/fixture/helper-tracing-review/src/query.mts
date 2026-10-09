// A type-only name must not replace the runtime String.raw built-in.
import type { String } from "./types.mjs";
import { write } from "@app/db";
import { sharedStatement, mutateSharedStatement, appendSharedStatement } from "./shared-state.mjs";
import { localAnnotatedStatement, localUnannotatedStatement } from "./local-tag.mjs";
import { diamondStatement } from "./diamond-barrel.mjs";
import { asyncLocalStatement } from "./async-local-tag.mjs";
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

export function importedModuleAppend() {
  write(appendSharedStatement()); // known:module-append
}
export function importedLocalTags() {
  write(localAnnotatedStatement()); // known:imported-local-tag
  write(localUnannotatedStatement()); // finding:imported-local-tag-missing
}
export function transparentExecutorWrappers() {
  void write(annotatedOrdersSql()); // known:unary-annotated
  void write(unannotatedOrdersSql()); // finding:unary-missing
  !write(annotatedOrdersSql()); // known:unary-not
  write(unannotatedOrdersSql()) === null; // finding:binary-missing
  (write(annotatedOrdersSql()), 1); // known:sequence-annotated
  (0, write(unannotatedOrdersSql())); // finding:sequence-missing
}

export async function importedAsyncLocalTag() {
  write(asyncLocalStatement()); // unanalyzable:unawaited-local-tag
  write(await asyncLocalStatement()); // known:awaited-local-tag
}

function executeLogical(statement: unknown) {
  write(statement); // finding:logical-callsite-conflict
}
export function logicalCallsites() {
  executeLogical(annotatedOrdersSql());
  flag && executeLogical(unannotatedOrdersSql());
}
function executeConditional(statement: unknown) {
  write(statement); // finding:conditional-callsite-conflict
}
export function conditionalCallsites() {
  executeConditional(annotatedOrdersSql());
  flag ? executeConditional(unannotatedOrdersSql()) : 0;
}
function executeOpaque(statement: unknown) {
  write(statement); // unanalyzable:unsupported-callsite-conflict
}
export function unsupportedCallsites() {
  executeOpaque(annotatedOrdersSql());
  if (flag) executeOpaque(unannotatedOrdersSql());
}

export function importedDiamondBinding() {
  write(diamondStatement()); // known:diamond-binding
}

// Unsupported lexical shadows must not invoke the same-name module helper.
function shadowedOpaqueExecutor(statement: unknown) {
  write(statement); // known:unmodeled-shadow-does-not-poison
}
export function modeledShadowedEntry() { shadowedOpaqueExecutor(annotatedOrdersSql()); }
export function unsupportedLexicalShadows(shadowedOpaqueExecutor: unknown, ...args: unknown[]) {
  if (flag) shadowedOpaqueExecutor(unannotatedOrdersSql());
}
export function unsupportedLoopShadow() {
  for (let n = 0, shadowedOpaqueExecutor = other; n < 1; n++) {
    if (flag) shadowedOpaqueExecutor(unannotatedOrdersSql());
  }
  for (const shadowedOpaqueExecutor of []) { shadowedOpaqueExecutor(); }
  for (const shadowedOpaqueExecutor in {}) { shadowedOpaqueExecutor(); }
  let shadowedOpaqueExecutor;
  for (; flag;) { shadowedOpaqueExecutor(); }
  for (shadowedOpaqueExecutor of []) { shadowedOpaqueExecutor(); }
  for (shadowedOpaqueExecutor in {}) { shadowedOpaqueExecutor(); }
}
export function unsupportedSwitchShadow() {
  switch (flag) {
    case true:
      const shadowedOpaqueExecutor = other;
      shadowedOpaqueExecutor(unannotatedOrdersSql());
  }
}
export function unsupportedCatchShadow() {
  try { throw 1; } catch (shadowedOpaqueExecutor) { shadowedOpaqueExecutor(); }
  try { throw 1; } catch { opaque(); }
}
namespace UnsupportedNamespace {
  const shadowedOpaqueExecutor = other;
  shadowedOpaqueExecutor();
}
class UnsupportedStatic {
  static { const shadowedOpaqueExecutor = other; shadowedOpaqueExecutor(); }
}
export const unsupportedNamedExpression = function shadowedOpaqueExecutor() {
  if (flag) shadowedOpaqueExecutor();
};
export const unsupportedArrowShadow = (shadowedOpaqueExecutor: unknown) => {
  if (flag) shadowedOpaqueExecutor();
};

function runOpaqueCallback(statement: unknown, callback: (value: unknown) => unknown) {
  return callback(statement);
}
function opaqueCallbackExecutor(statement: unknown) {
  write(statement); // unanalyzable:unsupported-callback-callsite-conflict
}
export function modeledCallbackEntry() { runOpaqueCallback(annotatedOrdersSql(), opaqueCallbackExecutor); }
if (flag) runOpaqueCallback(unannotatedOrdersSql(), opaqueCallbackExecutor);
if (flag) runOpaqueCallback(unannotatedOrdersSql(), (statement) => write(statement)); // unanalyzable:unsupported-inline-callback
if (flag) runOpaqueCallback(annotatedOrdersSql(), 0);

export function opaqueMethodReceiver() {
  const statement = sql`/* receiver before mutation */ SELECT 1`;
  const ignored = statement.mutate();
  write(statement); // unanalyzable:opaque-member-receiver
}
export function opaqueComputedReceiver() {
  const statement = sql`/* computed receiver */ SELECT 1`;
  const alias = statement;
  const ignored = alias[method]();
  write(statement); // unanalyzable:opaque-computed-receiver
}
export function optionalMethodReceiver() {
  const statement = sql`/* optional receiver */ SELECT 1`;
  const ignored = statement.mutate?.();
  write(statement); // unanalyzable:opaque-optional-receiver
}
export function spreadBuilderMutation() {
  const statement = sql`/* spread builder */ SELECT 1`;
  const ignored = unknownMutation(...[statement]);
  write(statement); // unanalyzable:opaque-spread-builder
}
export function spreadArrayBindingMutation() {
  const statement = sql`/* spread alias */ SELECT 1`;
  const aliases = [statement];
  const ignored = unknownMutation(...aliases);
  write(statement); // unanalyzable:opaque-spread-alias
}
function unsupportedSpreadPositions(first: unknown, statement: unknown) {
  write(statement); // unanalyzable:spread-positional-uncertainty
}
export function spreadPositions() {
  unsupportedSpreadPositions(...values, annotatedOrdersSql());
}
function spreadCallbackExecutor(statement: unknown) {
  write(statement); // unanalyzable:spread-callback-uncertainty
}
export function spreadCallbacks() {
  runOpaqueCallback(...[annotatedOrdersSql(), spreadCallbackExecutor]);
}
export function optionalHelperCall() {
  executeOptional(annotatedOrdersSql());
  executeOptional?.(unannotatedOrdersSql());
}
function executeOptional(statement: unknown) {
  write(statement); // unanalyzable:optional-helper-uncertainty
}

export function nestedSpreadBuilderMutation() {
  const statement = sql`/* nested spread */ SELECT 1`;
  const ignored = unknownMutation(...[...[statement]]);
  write(statement); // unanalyzable:nested-spread-builder
}
export function objectContainerBuilderMutation() {
  const statement = sql`/* object container */ SELECT 1`;
  const ignored = unknownMutation({statement});
  write(statement); // unanalyzable:object-container-builder
}
export function initializerPropertyAssignment() {
  const statement = sql`/* assignment initializer */ SELECT 1`;
  const ignored = (statement.text = 'SELECT 2');
  write(statement); // unanalyzable:initializer-property-write
}
export function initializerPropertyDelete() {
  const statement = sql`/* delete initializer */ SELECT 1`;
  const ignored = delete statement.text;
  write(statement); // unanalyzable:initializer-property-delete
}
export function opaqueConstructorMutation() {
  const statement = sql`/* constructor input */ SELECT 1`;
  const ignored = new unknownMutation(statement);
  write(statement); // unanalyzable:opaque-constructor-builder
}

export function deferredClassMethod() {
  const statement = sql`/* deferred method */ SELECT 1`;
  const Shell = class { unused() { unknownMutation(statement); } };
  write(statement); // known:class-method-deferred
}
export function initializerBindingAssignment() {
  let statement = sql`/* reassigned binding */ SELECT 1`;
  const ignored = (statement = unknownStatement);
  write(statement); // unanalyzable:initializer-binding-write
}
