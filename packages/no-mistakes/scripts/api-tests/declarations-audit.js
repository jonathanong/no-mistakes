const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("SQL recursive expression and INSERT lineage contracts export named additive types", () => {
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const expressions = readFileSync(join(packageRoot, "postgres-expression-types.d.ts"), "utf8");
  const inserts = readFileSync(join(packageRoot, "postgres-insert-types.d.ts"), "utf8");
  for (const name of [
    "PostgresSqlExpressionChild",
    "PostgresSqlExpressionChildRole",
    "PostgresSqlExpressionChildRoot",
  ])
    assert.match(expressions, new RegExp(`export (?:interface|type) ${name}\\b`));
  assert.match(source, /children: PostgresSqlExpressionChild\[\]/);
  assert.match(source, /childrenComplete: boolean/);
  assert.match(expressions, /role: PostgresSqlExpressionChildRole/);
  assert.match(expressions, /root: PostgresSqlExpressionChildRoot/);
  assert.match(expressions, /index: number \| null/);
  for (const name of [
    "PostgresSqlInsertColumnSources",
    "PostgresSqlInsertColumnSourcesReason",
    "PostgresSqlInsertColumnSource",
    "PostgresSqlInsertSourceExpression",
  ])
    assert.match(inserts, new RegExp(`export (?:interface|type) ${name}\\b`));
  assert.match(inserts, /columnSources: PostgresSqlInsertColumnSources/);
  assert.match(inserts, /columnIndex: number/);
  assert.match(inserts, /branchPath: number\[\]/);
  assert.match(inserts, /rowIndex: number/);
  assert.match(inserts, /expectedColumns\??: number/);
  assert.match(inserts, /sourceColumns\??: number/);
});

test("Postgres predicate and value discriminants remain public", () => {
  const expressions = readFileSync(join(packageRoot, "postgres-expression-types.d.ts"), "utf8");
  for (const declaration of [
    'kind: "nullTest"; negated: boolean',
    'kind: "distinctness"; negated: boolean',
    'kind: "parameter"; placeholder: string',
    'kind: "typedLiteral"; dataType: string; value: string; sql: string',
    "export type PostgresSqlLiteralValue =",
    'kind: "literal"; sql: string; value: PostgresSqlLiteralValue',
    '"nullOperand"',
    '"distinctLeft"',
    '"distinctRight"',
  ])
    assert.ok(expressions.includes(declaration), declaration);
});

test("declared payload configuration types are named public exports", () => {
  const types = readFileSync(join(packageRoot, "declared-payload-types.d.ts"), "utf8");
  const facade = readFileSync(join(packageRoot, "types.d.ts"), "utf8");
  for (const name of [
    "DeclaredPayloadCompatibilityOptions",
    "DeclaredPayloadContract",
    "DeclaredPayloadSchema",
  ]) {
    assert.ok(types.includes(`export interface ${name}`));
  }
  assert.ok(types.includes("contracts?: DeclaredPayloadContract[]"));
  assert.ok(types.includes("producer: DeclaredPayloadSchema"));
  assert.ok(types.includes("consumer: DeclaredPayloadSchema"));
  assert.ok(types.includes("pointer?: string"));
  assert.ok(facade.includes('export * from "./declared-payload-types"'));
});

test("reverse query declarations export bounded collection diagnostics", () => {
  const declarations = readFileSync(join(packageRoot, "query-types.d.ts"), "utf8");
  assert.match(declarations, /export type QueryIncompleteReason =/);
  for (const reason of ["operational-error", "parse-error", "missing-facts", "missing-symbols"]) {
    assert.ok(declarations.includes(`| "${reason}"`));
  }
  for (const name of ["QueryIncompleteFile", "QueryDiagnostics"]) {
    assert.match(declarations, new RegExp(`export interface ${name} \\{`));
  }
  for (const name of ["ImportersResult", "DeadExportsResult", "CallSitesResult"]) {
    assert.match(declarations, new RegExp(`export interface ${name} extends QueryDiagnostics`));
  }
  assert.match(declarations, /partial: boolean;/);
  assert.match(declarations, /partial\?: boolean;/);
  assert.match(declarations, /incompleteFiles: QueryIncompleteFile\[\];/);
  assert.match(declarations, /incompleteFiles\?: QueryIncompleteFile\[\];/);
});

test("audit declarations export named artifacts, evidence and async report contracts", () => {
  const declarations = readFileSync(join(packageRoot, "test-audit-types.d.ts"), "utf8");
  for (const name of [
    "TestAuditProvenance",
    "TestAuditSymbol",
    "TestAuditPlanArtifact",
    "TestAuditObservation",
    "TestAuditObservationsArtifact",
    "TestsAuditPlanFileInput",
    "TestsAuditPlanInlineInput",
    "TestsAuditObservationsFileInput",
    "TestsAuditObservationsInlineInput",
    "TestAuditExecutionEvidence",
    "TestAuditSelectionEvidence",
    "TestAuditReport",
  ]) {
    assert.match(declarations, new RegExp(`export interface ${name} \\{`));
  }
  assert.match(declarations, /granularity: "per-test-file";/);
  assert.match(declarations, /suite: "full";/);
  assert.match(declarations, /complete: true;/);
  assert.match(declarations, /changedSymbols: TestAuditSymbol\[\];/);
  assert.match(declarations, /selectedWithIncompleteTraces: string\[\];/);
  assert.match(
    declarations,
    /export type TestsAuditOptions = TestsAuditPlanInput & TestsAuditObservationsInput;/,
  );
  const api = readFileSync(join(packageRoot, "index-ci-infra.d.ts"), "utf8");
  assert.match(
    api,
    /export function testsAudit\(\s*options: WithInvocationOptions<TestsAuditOptions>,?\s*\): Promise<WithInvocationResult<TestAuditReport>>;/,
  );
  const aggregate = readFileSync(join(packageRoot, "analyze-project-types.d.ts"), "utf8");
  assert.match(aggregate, /type: "testsAudit"; id\?: string \} & TestsAuditOptions/);
});

test("scalar quantified query facts export typed operands and quantifier contracts", () => {
  const query = readFileSync(join(packageRoot, "postgres-query-types.d.ts"), "utf8");
  const expression = readFileSync(join(packageRoot, "postgres-expression-types.d.ts"), "utf8");
  assert.match(query, /quantifiedComparisons: PostgresSqlQueryQuantifiedComparison\[\];/);
  assert.match(query, /export type PostgresSqlQuantifier = "any" \| "all";/);
  assert.match(query, /export interface PostgresSqlQueryQuantifiedComparison \{/);
  for (const field of [
    "left: PostgresSqlExpression;",
    "right: PostgresSqlExpression;",
    "quantifier: PostgresSqlQuantifier;",
    "complete: boolean;",
  ])
    assert.ok(query.includes(field));
  assert.match(expression, /"arrayElement"/);
  assert.match(expression, /"array" \| "case"/);
});

// Lock each public subpath declaration, including named types and re-exports.
test("timeout policy declarations retain their complete public signatures", () => {
  const contracts = {
    "index.d.mts":
      'export { guardRegistration, type Registration, type Kind } from "./registration.mjs"; export { guardPublicRunner, type RunnerConstructor, type Protector } from "./public-runner.mjs"; export { assertTestTimeout, assertTimeoutConfig, snapshotTimeoutOptions, protectTimeoutConfig, } from "./config.mjs"; export declare function guardRuntimeConfig<T extends object>(api: T, maximum?: number): T; export declare function guardVitestExports<T extends object>(api: T, maximum?: number): T; export declare function protectRunnerTimeouts( runner: { config: object; }, maximum?: number, ): void;',
    "config.d.mts":
      "export declare function assertTestTimeout(value: unknown, label: string, maximum?: number): number; export declare function assertTimeoutConfig(config: object, maximum?: number): void; export declare function snapshotTimeoutOptions<T extends object>( options: T, label: string, maximum?: number, nullInherits?: boolean, ): T; export declare function protectTimeoutConfig(config: object, maximum?: number): void;",
    "registration.d.mts":
      'export type Registration = (...args: never[]) => unknown; export type Kind = "test" | "suite" | "hook"; export declare function guardRegistration<T extends Registration>( original: T, kind: Kind, maximum?: number, ): T;',
    "public-runner.d.mts":
      "export type Protector<T extends { config: object } = { config: object }> = ( runner: T, maximum: number, ) => void; export type RunnerConstructor = new (...args: never[]) => { config: object; }; export declare function guardPublicRunner<T extends RunnerConstructor>( original: T, maximum: number, protect: Protector<InstanceType<T>>, ): T;",
    "plugin.d.mts":
      'import type { Plugin } from "vitest/config"; export interface NodeRunnerPolicy { runner: string | null; } export declare function testTimeoutPolicyPlugin( maximum?: number, registrationPackages?: readonly string[], nodeRunner?: NodeRunnerPolicy, ): Plugin; export declare function browserTestTimeoutPolicyPlugin( maximum?: number, registrationPackages?: readonly string[], ): Plugin;',
  };
  for (const [file, expected] of Object.entries(contracts)) {
    const actual = readFileSync(join(packageRoot, "test-timeout-policy", file), "utf8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\s+/g, " ")
      .trim();
    assert.equal(actual, expected, file);
  }
});

// The consumer compiles the Vitest dependency graph; bound compiler work separately from policy deadlines.
