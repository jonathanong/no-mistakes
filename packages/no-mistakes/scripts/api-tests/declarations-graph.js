const { assert, test, readFileSync, join, packageRoot, repositoryRoot } = require("./context");

test("executed PostgreSQL source extraction exposes its named async contract", () => {
  const declarations = readFileSync(join(packageRoot, "executed-postgres-sql-types.d.ts"), "utf8");
  const index = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  const postgresSource = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  assert.match(declarations, /export interface ExecutedPostgresSource \{/);
  assert.match(declarations, /export interface ExecutedPostgresTrustedSqlTag \{/);
  assert.match(declarations, /export interface ExecutedPostgresResolvedImport \{/);
  assert.match(
    declarations,
    /export interface ExecutedPostgresSqlOptions<Diagnostics extends boolean = false> \{/,
  );
  assert.match(declarations, /trustedSqlTags\?: ExecutedPostgresTrustedSqlTag\[\];/);
  assert.match(declarations, /resolveImports\?: boolean;/);
  assert.match(declarations, /resolvedImports\?: ExecutedPostgresResolvedImport\[\];/);
  assert.match(declarations, /export interface ExecutedPostgresSql \{/);
  assert.match(declarations, /export type ExecutedPostgresSqlKind =/);
  assert.match(declarations, /kind: ExecutedPostgresSqlKind;/);
  assert.match(declarations, /complete: boolean;/);
  assert.match(declarations, /onlyValuePlaceholders: boolean;/);
  assert.match(declarations, /incompleteReason: string \| null;/);
  assert.match(
    declarations,
    /extractExecutedPostgresSql\(\s*options: ExecutedPostgresSqlOptions,\s*\): Promise<ExecutedPostgresSql\[\]>;/,
  );
  assert.match(
    index,
    /export \{ extractExecutedPostgresSql \} from "\.\/executed-postgres-sql-types";/,
  );
  assert.match(index, /export \* from "\.\/postgres-source-types";/);
  assert.match(postgresSource, /export \* from "\.\/executed-postgres-sql-types";/);
});

test(
  "executed PostgreSQL source extraction compiles from the package root",
  { timeout: 20_000 },
  () => {
    const { createRequire } = require("node:module");
    const dependencyRequire = createRequire(require.resolve("@storybook/react"));
    const ts = dependencyRequire("typescript");
    const program = ts.createProgram(
      [join(repositoryRoot, "fixtures", "node-api", "executed-postgres-sql", "public.mts")],
      {
        noEmit: true,
        strict: true,
        skipLibCheck: true,
        types: [],
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.NodeNext,
        moduleResolution: ts.ModuleResolutionKind.NodeNext,
        baseUrl: repositoryRoot,
        paths: { "no-mistakes": ["packages/no-mistakes/index.d.ts"] },
        ignoreDeprecations: "6.0",
      },
    );
    assert.deepEqual(
      ts
        .getPreEmitDiagnostics(program)
        .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
      [],
    );
  },
);

test("PostgreSQL metadata facts export routine signatures and typed index operations", () => {
  const declarations = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const metadata = readFileSync(join(packageRoot, "postgres-metadata-types.d.ts"), "utf8");
  assert.match(declarations, /export type \* from "\.\/postgres-metadata-types";/);
  assert.match(declarations, /kind: "comment"; comment: PostgresSqlComment/);
  assert.match(declarations, /kind: "alterIndex"; index: PostgresSqlAlterIndex/);
  assert.match(metadata, /export interface PostgresSqlComment/);
  assert.match(metadata, /arguments: PostgresSqlFunctionArgument\[\] \| null;/);
  assert.match(metadata, /comment: string \| null;/);
  assert.match(metadata, /export interface PostgresSqlAlterIndex/);
  assert.match(metadata, /ifExists: boolean;/);
  assert.match(metadata, /export type PostgresSqlAlterIndexOperation/);
  assert.match(metadata, /kind: "attachPartition"; partition: PostgresSqlName/);
  assert.match(metadata, /kind: "rename"; name: PostgresSqlName/);
});

test("resolveCheck declarations mirror its mutually exclusive runtime inputs", () => {
  const declarations = readFileSync(join(packageRoot, "query-types.d.ts"), "utf8");

  assert.match(declarations, /ResolveCheckOptions = QueryFileOptions & \{\n  files\?: never;/);
  assert.match(declarations, /files: \[string, \.\.\.string\[\]\];/);
  assert.match(declarations, /file\?: never;/);
  assert.match(declarations, /computed: boolean;/);
});

test("resolveConfig declarations expose additive per-framework triggers", () => {
  const declarations = readFileSync(join(packageRoot, "resolve-config-types.d.ts"), "utf8");

  assert.match(declarations, /vitestFullSuiteTriggers: ResolvedTrigger\[\];/);
  assert.match(declarations, /fullSuiteTriggers: ResolvedFrameworkTriggers\[\];/);
  assert.match(declarations, /includeChangedTests\?: boolean;/);
  assert.match(
    declarations,
    /export interface ResolvedFrameworkTriggers \{\n  framework: TestPlanFramework;\n  triggers: ResolvedTrigger\[\];\n\}/,
  );
  assert.match(declarations, /rewrites: ResolvedRewrite\[\];/);
  assert.match(declarations, /ignoreRoutes: string\[\];/);
  assert.match(
    declarations,
    /export interface ResolvedRewrite \{\n  source: string;\n  destination: string;\n\}/,
  );
});

test("route coverage declarations expose integration ownership and helper arguments", () => {
  const declarations = readFileSync(join(packageRoot, "resolve-config-types.d.ts"), "utf8");
  for (const name of ["RouteCoverageHelper", "RouteCoverageSource", "RouteCoverageAttribution"]) {
    assert.match(declarations, new RegExp(`export interface ${name} \\{`));
  }
  assert.match(declarations, /export type RouteCoverageFramework = "vitest";/);
  assert.equal(
    (declarations.match(/routeCoverageSources\?: RouteCoverageSource\[\];/g) || []).length,
    2,
  );
  assert.match(declarations, /urlArgument: number;/);
});

test("Vitest project policy declarations expose repository-relative setup files", () => {
  const declarations = readFileSync(join(packageRoot, "test-types.d.ts"), "utf8");
  assert.match(declarations, /export interface VitestProjectPolicy \{/);
  assert.match(declarations, /include\?: string\[\];/);
  assert.match(declarations, /setup_files\?: string\[\];/);
});

test("Swift package base commands are an exported argv map", () => {
  const declarations = readFileSync(join(packageRoot, "swift-command-types.d.ts"), "utf8");
  const targets = readFileSync(join(packageRoot, "test-types.d.ts"), "utf8");
  const barrel = readFileSync(join(packageRoot, "types.d.ts"), "utf8");
  assert.match(declarations, /export interface SwiftPackageBaseCommands \{/);
  assert.match(declarations, /\[packageRoot: string\]: string\[\];/);
  assert.match(targets, /Swift uses \{@link SwiftPackageBaseCommands\} or `\["swift", "test"\]`\./);
  assert.match(barrel, /export \* from "\.\/swift-command-types";/);
});

test("test plan declarations require current results but accept saved legacy plan documents", () => {
  const declarations = readFileSync(join(packageRoot, "test-types.d.ts"), "utf8");

  for (const pattern of [
    /export interface TestPlan \{\n  \/\*\* Complete deterministic changed-file inventory/,
    /\n  changedFiles: string\[\];/,
    /export type SavedTestPlan = TestPlan;/,
    /planJson\?: SavedTestPlan \| string;/,
    /export interface TestsWhyOptions \{[\s\S]*plan\?: string;\n  planJson\?: SavedTestPlan \| string;/,
    /export interface TestsPlanOptionsBase \{/,
    /export type TestsPlanOptions =/,
    /export type TestPlanFramework =\n  \| "vitest"\n  \| "playwright"\n  \| "dotnet"\n  \| "swift"\n  \| "python"\n  \| "go"\n  \| "cargo"\n  \| "rails"\n  \| "php"\n  \| "java"\n  \| "kotlin"\n  \| "elixir"\n  \| "dart"\n  \| "jest";/,
    /directTestOwner: true;[\s\S]*framework: TestPlanFramework;/,
    /framework: TestPlanFramework;[\s\S]*entrypoints\?: never;/,
    /limitPercent\?: never;[\s\S]*limitFiles\?: never;/,
    /globalConfigFallback\?: never;[\s\S]*directTestOwner\?: false;/,
    /export interface TestPlanIncompleteness \{/,
    /export type TestPlanIncompletenessKind = "dependency-baseline-missing";/,
    /incomplete\?: TestPlanIncompleteness\[\];/,
  ]) {
    assert.match(declarations, pattern);
  }
  assert.doesNotMatch(declarations, /TestPlan\["incomplete"\]/);
});

test("graph declarations expose GitHub Actions workflow relationships and virtual nodes", () => {
  const traversalDeclarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  const flowDeclarations = readFileSync(join(packageRoot, "flow-types.d.ts"), "utf8");

  for (const relationship of [
    "workflow",
    "workflow-job",
    "workflow-step",
    "workflow-needs",
    "workflow-uses",
    "workflow-run",
    "workflow-artifact",
  ]) {
    assert.match(traversalDeclarations, new RegExp(`\\| "${relationship}"`));
  }
  for (const pattern of [/workflowFile\?: string;/, /job\?: string;/, /step\?: number;/]) {
    assert.match(traversalDeclarations, pattern);
  }
  for (const pattern of [
    /"workflow-job"/,
    /"workflow-step"/,
    /workflowFile\?: string;/,
    /step\?: number;/,
  ]) {
    assert.match(flowDeclarations, pattern);
  }
});
