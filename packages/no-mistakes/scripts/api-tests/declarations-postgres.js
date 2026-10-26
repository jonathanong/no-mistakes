const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("UUID predicate rule exports its catalog and effort configuration", () => {
  const types = readFileSync(join(packageRoot, "postgres-uuid-predicate-types.d.ts"), "utf8");
  const exports = readFileSync(join(packageRoot, "types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresNoTransformedUuidPredicatesOptions/);
  assert.match(types, /schemaCatalogPath\?: string;/);
  assert.match(types, /effort\?: Effort;/);
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(types, /export type PostgresUuidPredicateUnanalyzableSql =/);
  assert.match(types, /unanalyzableSql\?: PostgresUuidPredicateUnanalyzableSql;/);
  assert.match(exports, /export type \* from "\.\/postgres-uuid-predicate-types"/);
});

test("generatePostgresCatalog declarations separate complete and ordering catalogs", () => {
  const types = readFileSync(join(packageRoot, "postgres-catalog-types.d.ts"), "utf8");
  const index = readFileSync(join(packageRoot, "index-ci-infra.d.ts"), "utf8");

  assert.match(types, /export type PostgresCatalogCoverage = "complete" \| "ordering";/);
  assert.match(types, /coverage\?: PostgresCatalogCoverage;/);
  assert.match(
    types,
    /export type PostgresSearchPathEvidence = Partial<Record<string, string\[\] \| null>>;/,
  );
  assert.match(types, /searchPathSchemas\?: string\[\];/);
  assert.match(types, /export interface PostgresCatalogOptions \{[^}]*currentDatabase\?: string;/);
  assert.equal((types.match(/searchPathEvidence\?: PostgresSearchPathEvidence;/g) || []).length, 2);
  assert.match(
    types,
    /export interface PostgresCompleteCatalog \{\n  formatVersion: 2;\n  coverage: "complete";/,
  );
  assert.match(
    types,
    /export interface PostgresOrderingCatalog \{\n  formatVersion: 2;\n  coverage: "ordering";/,
  );
  assert.match(types, /export type PostgresCatalog = /);
  for (const kind of ["Ordering", "Complete"]) {
    assert.match(
      types,
      new RegExp(`export interface Postgres${kind}Catalog \\{[^}]*currentDatabase\\?: string;`),
    );
  }
  for (const fact of ["functions", "enums", "views"]) {
    assert.match(types, new RegExp(`  ${fact}: Record<string, PostgresCatalog`));
  }
  assert.match(types, /export interface PostgresCatalogTable \{[^}]*partitionOf\?: string;/);
  for (const fact of ["foreignKeys", "checkConstraints", "triggers", "physicalPartition"]) {
    assert.match(types, new RegExp(`  ${fact}: `));
  }
  assert.match(
    index,
    /PostgresCatalogOptions & \{ coverage: "ordering" \}>,\n\): Promise<WithInvocationResult<PostgresOrderingCatalog>>;/,
  );
  assert.match(
    index,
    /PostgresCatalogOptions & \{ coverage\?: "complete" \}>,\n\): Promise<WithInvocationResult<PostgresCompleteCatalog>>;/,
  );
  assert.match(
    index,
    /WithInvocationOptions<PostgresCatalogOptions>,\n\): Promise<WithInvocationResult<PostgresCatalog>>;/,
  );
});

test("SQL expression roots expose named contracts without opaque AST dispatch", () => {
  const expressions = readFileSync(join(packageRoot, "postgres-expression-types.d.ts"), "utf8");
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  assert.match(expressions, /export type PostgresSqlExpressionRoot =/);
  assert.match(expressions, /export type PostgresSqlFunctionSyntax =/);
  assert.match(expressions, /export interface PostgresSqlCallArgument/);
  assert.match(expressions, /argumentsComplete: boolean/);
  assert.match(source, /root: PostgresSqlExpressionRoot/);
  assert.match(source, /export type \* from "\.\/postgres-expression-types"/);
});

test("parsePostgresSql exposes named pure-source contracts and async batch overloads", () => {
  const declarations = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const index = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  for (const type of [
    "PostgresSqlFacts",
    "PostgresSqlSource",
    "PostgresSqlColumn",
    "PostgresSqlConstraint",
    "PostgresSqlIndex",
    "PostgresSqlView",
    "PostgresSqlDiagnostic",
    "PostgresSqlPosition",
    "PostgresSqlSpan",
  ]) {
    assert.match(declarations, new RegExp(`export interface ${type} \\{`));
  }
  assert.match(index, /parsePostgresSql\(source: PostgresSqlSource\): Promise<PostgresSqlFacts>;/);
  assert.match(
    index,
    /parsePostgresSql\(sources: PostgresSqlSource\[\]\): Promise<PostgresSqlFacts\[\]>;/,
  );
  const functions = readFileSync(join(packageRoot, "postgres-function-types.d.ts"), "utf8");
  assert.match(functions, /export interface PostgresSqlFunctionArgument/);
  assert.match(functions, /export interface PostgresSqlFunction \{/);
  assert.match(declarations, /export type \* from "\.\/postgres-function-types";/);
  const drop = readFileSync(join(packageRoot, "postgres-drop-types.d.ts"), "utf8");
  assert.match(drop, /export interface PostgresSqlDrop/);
  assert.match(declarations, /export type \* from "\.\/postgres-drop-types";/);
  assert.match(declarations, /schemaVersion: 1;/);
  assert.match(declarations, /structuralIdentity: string;/);
  assert.match(declarations, /export interface PostgresSqlIndex \{[\s\S]*?only: boolean;/);
  assert.match(declarations, /functions: PostgresSqlFunctionReference\[\];/);
  assert.match(declarations, /kind: "validateConstraint"; name: PostgresSqlIdentifier/);
  assert.match(
    declarations,
    /export interface PostgresSqlConstraint \{[^}]*span: PostgresSqlSpan \| null;/,
  );
  assert.match(declarations, /dependenciesComplete: boolean;/);
  assert.match(declarations, /kind: "doBlock"; block: PostgresSqlProceduralBlock/);
  const procedural = readFileSync(join(packageRoot, "postgres-procedural-types.d.ts"), "utf8");
  assert.match(procedural, /export interface PostgresSqlProceduralBlock/);
  assert.match(procedural, /bodySpan: PostgresSqlSpan;/);
  assert.match(procedural, /bodyEncoding: PostgresSqlBodyEncoding;/);
  assert.match(
    procedural,
    /export type PostgresSqlBodyEncoding = "dollarQuoted" \| "singleQuoted" \| "escapedString";/,
  );
  assert.match(procedural, /export interface PostgresSqlConditionalBranch/);
  assert.match(procedural, /condition: PostgresSqlExpression \| null;/);
  assert.match(declarations, /kind: "conditional";\s+branches: PostgresSqlConditionalBranch\[\]/);
  assert.match(procedural, /statements: PostgresSqlStatement\[\];/);
  assert.match(procedural, /complete: boolean;/);
  assert.match(procedural, /supported static SQL is reachable through typed statements/);
  assert.match(procedural, /Unprojected loop utilities produce diagnostics and keep this false/);
  assert.match(
    procedural,
    /Recognized utilities represented as `other` statements retain coverage/,
  );
  assert.match(procedural, /Occurrence classification is additive and never proves typed coverage/);
  assert.match(procedural, /occurrences: PostgresSqlProceduralOccurrence\[\];/);
  assert.match(procedural, /export interface PostgresSqlProceduralOccurrence/);
  assert.match(
    procedural,
    /export type PostgresSqlProceduralOccurrenceKind =\n {2}\| "utility"\n {2}\| "controlFlow"\n {2}\| "dml"\n {2}\| "dynamicExecute"\n {2}\| "unknown";/,
  );
});
