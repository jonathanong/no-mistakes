import { write } from "@app/db";
import sql, { type SQLStatement } from "sql-template-strings";
import { importedOrdersSql } from "./sql-builders.mjs";

export async function direct() {
  return write(sql`SELECT 1`); // finding:direct
}

function pendingOrdersSql(select: string): SQLStatement {
  return sql``.append(`SELECT ${select} FROM orders WHERE shipped_at IS NULL`);
}

export async function helperExecutedDirectly(id: string) {
  return write(pendingOrdersSql("id").append(sql` AND id = ${id}`)); // finding:helper-direct
}

function annotatedOrdersSql(): SQLStatement {
  return sql`/* getPendingOrders */ SELECT id FROM orders WHERE shipped_at IS NULL`;
}

export async function annotatedHelperPasses() {
  return write(annotatedOrdersSql()); // known:annotated-helper
}

export async function callerPrependsAnnotation() {
  return write(sql`/* callerNamedQuery */ `.append(pendingOrdersSql("id"))); // known:caller-prefix
}

export async function importedHelperExecutedDirectly() {
  return write(importedOrdersSql()); // finding:imported-helper
}

function unknownFirstSql(prefix: string): SQLStatement {
  return sql``.append(prefix).append(sql`/* too late */ SELECT id FROM orders`);
}

export async function unknownAppendBeforeAnnotation(prefix: string) {
  return write(unknownFirstSql(prefix)); // unanalyzable:unknown-first
}

function annotationFirstSql(suffix: string): SQLStatement {
  return sql`/* known first */ SELECT id FROM orders`.append(suffix);
}

export async function annotationBeforeUnknownAppend(suffix: string) {
  return write(annotationFirstSql(suffix)); // known:annotation-first
}

function dynamicOrdersSql(statement: string): SQLStatement {
  return sql``.append(statement);
}

export async function unresolvedBuilder(prefix: string) {
  return write(dynamicOrdersSql(prefix)); // unanalyzable:dynamic-builder
}

async function pageIds(statement: SQLStatement, run: (value: SQLStatement) => Promise<unknown>) {
  return run(statement);
}

export async function helperExecutedThroughInlineCallback() {
  return pageIds(pendingOrdersSql("id"), (statement) => write(statement)); // finding:inline-callback
}

const runCallbackKnownFirst = (statement: SQLStatement) => write(statement); // finding:callback-known-first
const runCallbackUnknownFirst = (statement: SQLStatement) => write(statement); // finding:callback-unknown-first

export async function callbackCallsitesKnownThenUnknown() {
  return Promise.all([
    pageIds(annotatedOrdersSql(), runCallbackKnownFirst),
    pageIds(pendingOrdersSql("id"), runCallbackKnownFirst),
  ]);
}

export async function callbackCallsitesUnknownThenKnown() {
  return Promise.all([
    pageIds(pendingOrdersSql("id"), runCallbackUnknownFirst),
    pageIds(annotatedOrdersSql(), runCallbackUnknownFirst),
  ]);
}
