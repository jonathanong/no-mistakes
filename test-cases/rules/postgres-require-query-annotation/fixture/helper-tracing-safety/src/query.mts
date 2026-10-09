import { write } from "@app/db";
import templateSql, { type SQLStatement } from "sql-template-strings";
import sqlTemplate from "sql-template-strings";
import { customQuery, unknownQuery as sql } from "./tags.mjs";
import { reexportedOrdersSql } from "./reexports.mjs";
import * as builders from "./builders.mjs";

function annotatedOrdersSql(): SQLStatement {
  return templateSql`/* safe */ SELECT id FROM orders`;
}

function unannotatedOrdersSql(): SQLStatement {
  return templateSql`SELECT id FROM orders`;
}

export async function reassignedLocalHelper() {
  let builder = annotatedOrdersSql;
  builder = unannotatedOrdersSql;
  return write(builder()); // unanalyzable:reassigned-helper
}

export async function helperParameterShadowsModuleHelper(
  annotatedOrdersSql: () => SQLStatement,
) {
  return write(annotatedOrdersSql()); // unanalyzable:shadow-helper
}

export async function parameterShadowsTrustedTag(
  templateSql: (strings: TemplateStringsArray) => SQLStatement,
) {
  return write(templateSql`SELECT id FROM orders`); // unanalyzable:shadow-tag
}

function cycleA(): SQLStatement {
  return cycleB();
}

function cycleB(): SQLStatement {
  return cycleA();
}

export async function recursiveHelperCycle() {
  return write(cycleA()); // unanalyzable:cycle
}

function unsupportedBranch(flag: boolean): SQLStatement {
  if (flag) return annotatedOrdersSql();
  return unannotatedOrdersSql();
}

export async function branchIsNotAssumedSafe(flag: boolean) {
  return write(unsupportedBranch(flag)); // unanalyzable:branch
}

function restParameterHelper(...parts: string[]): SQLStatement {
  return sql``.append(parts[0]);
}

export async function restParameterIsNotAssumedSafe() {
  return write(restParameterHelper("id")); // unanalyzable:rest-parameter
}

function destructuredParameterHelper({ column }: { column: string }): SQLStatement {
  return sql``.append(column);
}

export async function destructuredParameterIsNotAssumedSafe() {
  return write(destructuredParameterHelper({ column: "id" })); // unanalyzable:destructured-parameter
}

const reassignedCallback = (statement: SQLStatement) => {
  statement = annotatedOrdersSql();
  return write(statement); // unanalyzable:callback-param-reassignment
};

export async function callbackParameterReassignmentIsOpaque() {
  return reassignedCallback(sql`/* safe */ SELECT id FROM orders`);
}

export async function trustedDefaultTagAliasPasses() {
  return write(templateSql`/* safe */ SELECT id FROM orders`); // known:default-tag
}

export async function configuredTagPasses() {
  return write(customQuery`/* custom */ SELECT id FROM orders`); // known:configured-tag
}

export async function importedReExportPasses() {
  return write(reexportedOrdersSql()); // known:import-re-export
}

export async function namespaceHelperIsOpaque() {
  return write(builders.annotatedOrdersSql()); // unanalyzable:namespace-helper
}

export async function untrustedSqlNamedImportLooksAnnotated() {
  return write(sql`/* not trusted */ SELECT id FROM orders`); // unanalyzable:untrusted-sql-import
}

// A configured trusted import that is reassigned is no longer usable as proof.
sqlTemplate = customQuery;

export async function reassignedTrustedTagLooksAnnotated() {
  return write(sqlTemplate`/* no longer trusted */ SELECT id FROM orders`); // unanalyzable:reassigned-trusted-tag
}

export async function shadowedStringRawIsOpaque(
  String: { raw(strings: TemplateStringsArray): SQLStatement },
) {
  return write(String.raw`/* looks named */ SELECT id FROM orders`); // unanalyzable:shadowed-string-raw
}

export async function trustedStringRawPasses() {
  return write(String.raw`/* raw */ SELECT id FROM orders`); // known:string-raw
}

export async function invalidCookedTaggedTemplateStillReports() {
  return write(templateSql`\xZ`); // finding:invalid-cooked-template
}

function concatenatedQuery(column: string): SQLStatement {
  return templateSql`SELECT ${column} FROM orders` + " WHERE archived = false";
}

export async function concatenatedTemplateHasKnownUnannotatedPrefix() {
  return write(concatenatedQuery("id")); // finding:binary-template
}

function appendIntoLocalBuilder(): SQLStatement {
  const statement = templateSql``;
  statement.append("SELECT id FROM orders");
  return statement;
}

export async function appendStatementIsTraced() {
  return write(appendIntoLocalBuilder()); // finding:local-append
}

function mutateAndReturn(statement: SQLStatement): SQLStatement {
  externalMutation(statement);
  return statement;
}

export async function effectfulReturningHelperIsOpaque() {
  return write(mutateAndReturn(templateSql`/* looks safe */ SELECT id FROM orders`)); // unanalyzable:effectful-return
}

function effectfulTail(): string {
  externalEffect();
  return " WHERE archived = false";
}

export async function callerAnnotationSurvivesEffectfulAppend() {
  return write(templateSql`/* caller prefix */ SELECT id FROM orders`.append(effectfulTail())); // known:caller-prefix
}

export async function chainedAppendPreservesKnownHeader() {
  const statement = templateSql``;
  statement.append(templateSql`/* chained prefix */ SELECT id FROM orders`).append(opaqueValue);
  return write(statement); // known:chained-append-prefix
}

const namedRecursiveBuilder = function namedRecursiveBuilder(): SQLStatement {
  return namedRecursiveBuilder();
};

export async function namedFunctionExpressionCycleIsOpaque() {
  return write(namedRecursiveBuilder()); // unanalyzable:named-function-cycle
}

export async function arrayArgumentIsOpaque() {
  return write([templateSql`/* nested */ SELECT id FROM orders`]); // unanalyzable:array-argument
}

function makeBuilder(): () => SQLStatement {
  return unknownBuilder();
}

export async function dynamicCalleeIsOpaque() {
  return write(makeBuilder()()); // unanalyzable:dynamic-callee
}

const localMethods = { build: annotatedOrdersSql };

export async function memberCalleeIsOpaque() {
  return write(localMethods.build()); // unanalyzable:member-callee
}

export async function destructuredLocalBindingIsOpaque() {
  const { column } = { column: "id" };
  return write(templateSql``.append(column)); // unanalyzable:destructured-local
}
