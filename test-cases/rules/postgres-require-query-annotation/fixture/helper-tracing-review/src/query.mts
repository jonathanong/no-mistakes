// A type-only name must not replace the runtime String.raw built-in.
import type { String } from './types.mjs';
import { write } from "@app/db";
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
