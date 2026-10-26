require("./api-tests/proxy");
require("./api-tests/planning");
require("./api-tests/exports");
require("./api-tests/declarations-core");
require("./api-tests/declarations-unbounded");
require("./api-tests/declarations-export-invariants");
require("./api-tests/declarations-mutable");
require("./api-tests/declarations-test-state-mutation");
require("./api-tests/declarations-postgres");
require("./api-tests/declarations-graph");
require("./api-tests/declarations-executor-diagnostics");
require("./api-tests/declarations-analysis");
require("./api-tests/declarations-sql");
require("./api-tests/declarations-audit");
require("./api-tests/declarations-runtime");

const { assert, test, readFileSync, join, packageRoot } = require("./api-tests/context");
test("forbidden dependency selectors and origin exceptions have public option types", () => {
  const declarations = readFileSync(join(packageRoot, "forbidden-dependency-types.d.ts"), "utf8");
  assert.match(declarations, /export interface ForbiddenDependencyException \{/);
  assert.match(declarations, /export interface ForbiddenDependenciesOptions \{/);
  assert.match(declarations, /roots: string\[\];/);
  assert.match(declarations, /excludeRoots\?: string\[\];/);
  assert.match(declarations, /forbiddenModules\?: string\[\];/);
  assert.match(declarations, /forbiddenFiles\?: string\[\];/);
  assert.match(declarations, /relationships\?: Relationship\[\];/);
  assert.match(declarations, /exceptions\?: ForbiddenDependencyException\[\];/);
  assert.match(declarations, /allowedFiles: string\[\];/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/forbidden-dependency-types";/,
  );
});

test("annotation literal helpers remain an exported option and verdict profile", () => {
  for (const file of ["postgres-query-annotation-types.d.ts", "check-report-types.d.ts"]) {
    assert.match(readFileSync(join(packageRoot, file), "utf8"), /sqlLiteralHelpers\?: string\[\];/);
  }
});

test("Playwright coverage exposes named optional matcher warnings", () => {
  const declarations = readFileSync(join(packageRoot, "report-types.d.ts"), "utf8");
  assert.match(declarations, /export interface PlaywrightCoverageReport/);
  assert.match(declarations, /warnings\?: string\[\]/);
  const index = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  assert.match(index, /playwrightCheck\([\s\S]*?Promise<PlaywrightCoverageReport>/);
});

test("coverage assertion owner options are public and bounded", () => {
  const declaration = readFileSync(
    join(packageRoot, "coverage-assertion-owner-types.d.ts"),
    "utf8",
  );
  for (const name of [
    "CoverageProducerFunction",
    "CoverageAssertionImport",
    "CoverageAssertionOwnership",
    "CoverageProducerAssertionOwnerOptions",
  ]) {
    assert.match(declaration, new RegExp("export interface " + name));
  }
  for (const field of [
    "producers:",
    "assertions:",
    "ownership?:",
    "maxDepth?:",
    "maxNodes?:",
    "unanalyzable?:",
  ])
    assert.ok(declaration.includes(field));
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      "./coverage-assertion-owner-types",
    ),
  );
});

test("forbidden lint suppression options expose exact directive choices", () => {
  const declaration = readFileSync(join(packageRoot, "lint-suppression-types.d.ts"), "utf8");
  assert.match(declaration, /export type ForbiddenLintSuppressionDirective\s*=/);
  assert.match(declaration, /export interface ForbiddenLintSuppressionsOptions/);
  for (const directive of [
    "eslint-disable",
    "eslint-disable-line",
    "eslint-disable-next-line",
    "oxlint-disable",
    "oxlint-disable-line",
    "oxlint-disable-next-line",
    "@ts-ignore",
    "@ts-expect-error",
  ]) {
    assert.ok(declaration.includes(`"${directive}"`));
  }
  for (const field of [
    "files: string[];",
    "ruleIds?: string[];",
    "rulePrefixes?: string[];",
    "directives?: ForbiddenLintSuppressionDirective[];",
    "banBlanketDisable?: boolean;",
    "message?: string;",
  ]) {
    assert.ok(declaration.includes(field));
  }
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes("./lint-suppression-types"),
  );
});

test("forbidden lint suppression configuration compiles from the package root", () => {
  const { createRequire } = require("node:module");
  const dependencyRequire = createRequire(require.resolve("@storybook/react"));
  const ts = dependencyRequire("typescript");
  const repositoryRoot = join(packageRoot, "..", "..");
  const program = ts.createProgram(
    [join(repositoryRoot, "fixtures/node-api/forbidden-lint-suppressions/public.mts")],
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
      .map((row) => ts.flattenDiagnosticMessageText(row.messageText, "\n")),
    [],
  );
});
