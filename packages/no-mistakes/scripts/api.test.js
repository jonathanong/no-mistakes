require("./api-tests/declarations-enqueue-patterns");
require("./api-tests/declarations-download-integrity");
require("./api-tests/declarations-pnpm-release-age");
require("./api-tests/declarations-diagnostics");
require("./api-tests/diagnostics");
require("./api-tests/proxy");
require("./api-tests/planning");
require("./api-tests/exports");
require("./api-tests/invocation-errors");
require("./api-tests/declarations-core");
require("./api-tests/declarations-lockfile");
require("./api-tests/declarations-unbounded");
require("./api-tests/declarations-export-invariants");
require("./api-tests/declarations-mutable");
require("./api-tests/declarations-test-state-mutation");
require("./api-tests/declarations-live-repository-scan");
require("./api-tests/declarations-postgres");
require("./api-tests/postgres-lock-inventory");
require("./api-tests/declarations-grouping");
require("./api-tests/declarations-duplicate-keyed-lookups");
require("./api-tests/declarations-graph");
require("./api-tests/declarations-queue-factories");
require("./api-tests/declarations-executor-diagnostics");
require("./api-tests/declarations-analysis");
require("./api-tests/declarations-sql");
require("./api-tests/declarations-audit");
require("./api-tests/declarations-runtime");

const {
  assert,
  test,
  readFileSync,
  join,
  packageRoot,
  assertFixturesCompile,
} = require("./api-tests/context");

test("cache setup policy exports both workflow selectors", () => {
  const options = readFileSync(
    join(packageRoot, "github-actions-cache-setup-policy-types.d.ts"),
    "utf8",
  );
  assert.match(options, /export interface GitHubActionsCacheSetupPolicyOptions/);
  assert.match(options, /export interface GitHubActionsCacheKeyShape/);
  assert.match(options, /export interface GitHubActionsSetupDeadline/);
  assert.match(options, /unanalyzable\?: Unanalyzable;/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-cache-setup-policy-types";/,
  );
});

test("github-actions-pinned-hash images option is publicly named", () => {
  const declaration = readFileSync(
    join(packageRoot, "github-actions-pinned-hash-types.d.ts"),
    "utf8",
  );
  assert.match(declaration, /export interface GithubActionsPinnedHashOptions \{/);
  assert.match(declaration, /images\?: boolean;/);
  assert.match(declaration, /unanalyzable\?: Unanalyzable;/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-pinned-hash-types";/,
  );
});

test("module mock lint options name the keys each rule reads", () => {
  const declarations = readFileSync(
    join(packageRoot, "..", "eslint-plugin-no-mistakes", "src", "index.d.ts"),
    "utf8",
  );
  assert.match(declarations, /export interface ModuleMockIntegrationExports \{/);
  const boundary = declarations.match(/export interface ModuleMockBoundaryOptions \{([^}]+)\}/);
  assert.ok(boundary, "ModuleMockBoundaryOptions must be exported");
  assert.match(boundary[1], /baseline\?: \[string, string, number\]\[\];/);
  assert.match(boundary[1], /requireLiteralSpecifiers\?: boolean;/);
  assert.match(boundary[1], /integrationExports\?: ModuleMockIntegrationExports;/);
  assert.match(boundary[1], /unanalyzable\?: RuleUnanalyzable;/);
  const preserve = declarations.match(
    /export interface ModuleMockPreserveExportsOptions \{([^}]+)\}/,
  );
  assert.ok(preserve, "ModuleMockPreserveExportsOptions must be exported");
  assert.match(preserve[1], /baseline\?: \[string, string\]\[\];/);
  assert.match(preserve[1], /internalSpecifiers\?: string\[\];/);
  assert.match(preserve[1], /unanalyzable\?: RuleUnanalyzable;/);
});

test("typed enqueue fake lint options are publicly declared", () => {
  const declarations = readFileSync(
    join(packageRoot, "..", "eslint-plugin-no-mistakes", "src", "index.d.ts"),
    "utf8",
  );
  const options = declarations.match(/export interface TestNoTypedEnqueueFakeOptions \{([^}]+)\}/);
  assert.ok(options, "TestNoTypedEnqueueFakeOptions must be exported");
  assert.match(options[1], /singleExportPattern\?: string;/);
  assert.match(options[1], /moduleGlobs\?: string\[\];/);
});

test("postgres eslint options name the unanalyzable policy", () => {
  const declarations = readFileSync(
    join(packageRoot, "..", "eslint-plugin-no-mistakes", "src", "index.d.ts"),
    "utf8",
  );
  assert.match(
    declarations,
    /export type PostgresUnanalyzable = "advise" \| "report" \| "ignore";/,
  );
  const adapterOptions = declarations.match(
    /export interface PostgresNoExecutorAdapterOptions \{([\s\S]*?)\n  \}/,
  );
  assert.ok(adapterOptions, "PostgresNoExecutorAdapterOptions must be exported");
  assert.match(adapterOptions[1], /unanalyzable\?: PostgresUnanalyzable;/);
  assert.match(
    declarations,
    /export interface PostgresPoolRequiresErrorListenerOptions \{[\s\S]*?unanalyzable\?: PostgresUnanalyzable;/,
  );
});

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

test("action timeout pair exports the optional nested deadline comparison", () => {
  const declarations = readFileSync(
    join(packageRoot, "github-actions-action-timeout-pair-types.d.ts"),
    "utf8",
  );
  assert.match(declarations, /export interface GithubActionsActionTimeoutPairOptions \{/);
  assert.match(declarations, /nestedBelowStepTimeout\?: boolean;/);
  assert.match(declarations, /unanalyzable\?: Unanalyzable;/);
});

test("annotation literal helpers remain an exported option and verdict profile", () => {
  for (const file of ["postgres-query-annotation-types.d.ts", "check-report-types.d.ts"]) {
    assert.match(readFileSync(join(packageRoot, file), "utf8"), /sqlLiteralHelpers\?: string\[\];/);
  }
});

test("check reports expose named prerequisite failure contracts", () => {
  const declarations = readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8");
  assert.match(
    declarations,
    /export interface CheckFailure \{[\s\S]*?consumer: string;[\s\S]*?file\?: string;[\s\S]*?property\?: string;[\s\S]*?message: string;/,
  );
  assert.match(declarations, /incomplete\?: CheckFailure\[\];/);
  assert.match(
    readFileSync(join(packageRoot, "report-types.d.ts"), "utf8"),
    /export \* from "\.\/check-report-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/report-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export \* from "\.\/types";/,
  );
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

test("Dependabot cooldown policy has exported option types", () => {
  const declaration = readFileSync(
    join(packageRoot, "dependabot-cooldown-policy-types.d.ts"),
    "utf8",
  );
  assert.match(declaration, /export interface DependabotCooldownPolicyOptions/);
  assert.match(declaration, /minimumDays: number;/);
  assert.match(declaration, /requireOpenPullRequestsLimit\?: boolean;/);
  assert.match(declaration, /allowedCooldownExcludes\?: string\[\];/);
  assert.match(declaration, /exemptEcosystems\?: string\[\];/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/dependabot-cooldown-policy-types";/,
  );
});

test("shell compatibility rules expose optional path globs", () => {
  const declaration = readFileSync(join(packageRoot, "shell-compatibility-types.d.ts"), "utf8");
  for (const name of ["BashEmptyArrayNounsetOptions", "ZshReadonlyParameterOptions"]) {
    assert.match(
      declaration,
      new RegExp(`export interface ${name} \\{[^}]*files\\?: string\\[\\];`),
    );
  }
});

test("check rule options compile from the package root", () => {
  const repositoryRoot = join(packageRoot, "..", "..");
  assertFixturesCompile(
    [
      join(repositoryRoot, "fixtures/node-api/dependabot-cooldown-policy/public.mts"),
      join(repositoryRoot, "fixtures/node-api/forbidden-lint-suppressions/public.mts"),
      join(repositoryRoot, "fixtures/node-api/github-actions-env-policy/public.mts"),
      join(repositoryRoot, "fixtures/node-api/github-actions-no-poll-wait-job/public.mts"),
      join(repositoryRoot, "fixtures/node-api/github-actions-run-command-policy/public.mts"),
      join(repositoryRoot, "fixtures/node-api/postgres-lock-inventory/public.mts"),
      join(repositoryRoot, "fixtures/node-api/postgres-sql-statement-policy/public.mts"),
      join(repositoryRoot, "fixtures/node-api/pnpm-release-age-policy/public.mts"),
      join(repositoryRoot, "fixtures/node-api/shell-compatibility/public.mts"),
      join(repositoryRoot, "fixtures/node-api/test-fixture-size-cap/public.mts"),
      join(repositoryRoot, "fixtures/node-api/test-reliability/public.mts"),
      join(repositoryRoot, "fixtures/node-api/vitest-default-excludes/public.mts"),
      join(repositoryRoot, "fixtures/node-api/vitest-project-policy/public.mts"),
    ],
    (ts) => ({
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
    }),
  );
});

test("test-no-retries exposes optional config factory path globs and fixture exports", () => {
  const declaration = readFileSync(join(packageRoot, "test-reliability-types.d.ts"), "utf8");
  assert.match(
    declaration,
    /export interface PlaywrightFixtureExport \{[^}]*module: string;[^}]*export: string;/s,
  );
  assert.match(declaration, /export interface TestNoRetriesOptions \{[^}]*files\?: string\[\];/);
  assert.match(
    declaration,
    /export interface TestNoRetriesOptions \{[^}]*fixtures\?: PlaywrightFixtureExport\[\];/,
  );
  assert.match(
    declaration,
    /export interface PlaywrightNoSerialOptions \{[^}]*fixtures\?: PlaywrightFixtureExport\[\];/,
  );
});

test("production export test-only options require explicit production and test scope", () => {
  const declaration = readFileSync(
    join(packageRoot, "production-exports-test-only-types.d.ts"),
    "utf8",
  );
  assert.match(declaration, /export interface ProductionExportsOnlyUsedByTestsOptions/);
  assert.match(declaration, /productionRoots: string\[\];/);
  assert.match(declaration, /testGlobs\?: string\[\];/);
  assert.match(declaration, /testHelperRoots\?: string\[\];/);
  assert.match(declaration, /ignoreExportsUsedInFile\?: boolean;/);
  assert.match(declaration, /export interface PlatformConsumedExports/);
  assert.match(declaration, /platformConsumedExports\?: PlatformConsumedExports\[\];/);
  assert.match(declaration, /unanalyzable\?: Unanalyzable;/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      "./production-exports-test-only-types",
    ),
  );
});

test("bounded statement options export key fan-out contracts", () => {
  const types = readFileSync(join(packageRoot, "postgres-bounded-statements-types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresBoundedStatementsOptions/);
  assert.match(types, /export interface PostgresColumnBound \{\s*reason: string;/);
  assert.match(types, /export type PostgresViewBoundMode = "bounded" \| "unbounded";/);
  assert.match(types, /columnBounds\?: Record<string, PostgresColumnBound>;/);
  assert.match(
    types,
    /export type PostgresMaterializedBodyPolicy = "ignore" \| "advise" \| "report" \| "require-bound";/,
  );
  assert.match(types, /materializedBodies\?: PostgresMaterializedBodyPolicy;/);
  assert.match(types, /viewBounds\?: Record<string, PostgresViewBoundMode>;/);
  assert.match(types, /maxDomainRows\?: number;/);
  assert.match(types, /schemaCatalogPath: string;/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-bounded-statements-types";',
    ),
  );
});

test("identifier assertion contracts are shared by supported PostgreSQL rules", () => {
  const types = readFileSync(
    join(packageRoot, "postgres-sql-identifier-assertion-types.d.ts"),
    "utf8",
  );
  assert.match(types, /export interface PostgresSqlIdentifierAssertion/);
  assert.match(types, /module: string;/);
  assert.match(types, /name: string;/);
  assert.match(types, /allowlistArgument: number;/);
  for (const file of [
    "postgres-test-state-mutation-types.d.ts",
    "postgres-bounded-statements-types.d.ts",
  ]) {
    const options = readFileSync(join(packageRoot, file), "utf8");
    assert.match(options, /import type \{ PostgresSqlIdentifierAssertion \} from/);
    assert.match(options, /sqlIdentifierAssertions\?: PostgresSqlIdentifierAssertion\[\];/);
  }
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-sql-identifier-assertion-types";',
    ),
  );
});

test("raw SQL tag contracts are shared by supported PostgreSQL rules", () => {
  const types = readFileSync(join(packageRoot, "postgres-raw-sql-tag-types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresRawSqlTag/);
  assert.match(types, /module: string;/);
  assert.match(types, /name: string;/);
  for (const file of [
    "postgres-test-state-mutation-types.d.ts",
    "postgres-bounded-statements-types.d.ts",
  ]) {
    const options = readFileSync(join(packageRoot, file), "utf8");
    assert.match(options, /import type \{ PostgresRawSqlTag \} from/);
    assert.match(options, /rawSqlTags\?: PostgresRawSqlTag\[\];/);
  }
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-raw-sql-tag-types";',
    ),
  );
});

test("SQL executor profiles expose rule-local identifier and raw tag contracts", () => {
  const report = readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8");
  assert.match(report, /import type \{ PostgresSqlIdentifierAssertion \} from/);
  assert.match(report, /import type \{ PostgresRawSqlTag \} from/);
  assert.match(report, /sqlIdentifierAssertions\?: PostgresSqlIdentifierAssertion\[\];/);
  assert.match(report, /rawSqlTags\?: PostgresRawSqlTag\[\];/);
});

test("signature impact exposes scoped unanalyzed files", () => {
  const impact = readFileSync(join(packageRoot, "signature-impact-types.d.ts"), "utf8");
  assert.match(impact, /export interface UnanalyzedFile \{\s*file: string;\s*reason: string;/);
  assert.match(impact, /unanalyzed: UnanalyzedFile\[\];/);
  assert.match(readFileSync(join(packageRoot, "types.d.ts"), "utf8"), /traversal-types/);
});

test("accepted statement bounds are an exported check report field", () => {
  const report = readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8");
  assert.match(report, /export interface AcceptedStatementBound \{/);
  assert.match(report, /acceptedBounds\?: AcceptedStatementBound\[\];/);
  assert.match(report, /rows: number;/);
  assert.match(report, /justification: string;/);
});

test("check timings have named public types", () => {
  const report = readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8");
  const options = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  assert.match(report, /export interface CheckTiming \{/);
  assert.match(report, /timings\?: CheckTiming\[\]/);
  assert.match(options, /timings\?: boolean/);
});

test("per-item query configuration exports optional capped cursor exemption", () => {
  const declarations = readFileSync(join(packageRoot, "query-reached-per-item-types.d.ts"), "utf8");
  assert.match(declarations, /export interface QueryReachedPerItemOptions/);
  assert.match(declarations, /effects: string\[\];/);
  assert.match(declarations, /allow\?: QueryReachedPerItemAllowEntry\[\];/);
  assert.match(declarations, /exemptCappedCursorPageReads\?: boolean;/);
  assert.match(declarations, /export interface QueryReachedPerItemAllowEntry/);
  assert.match(declarations, /file: string;/);
  assert.match(declarations, /function\?: string;/);
  assert.match(declarations, /callee\?: string;/);
  assert.match(declarations, /line\?: number;/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./query-reached-per-item-types";',
    ),
  );
});

test("declared config exports its unused allow-entry policy", () => {
  const declarations = readFileSync(join(packageRoot, "resolve-config-types.d.ts"), "utf8");
  assert.match(
    declarations,
    /export type UnusedAllowEntryPolicy = "advise" \| "report" \| "ignore";/,
  );
  assert.match(
    declarations,
    /export interface DeclaredConfig extends DeclaredConfigObject \{[\s\S]*?reportUnusedAllowEntries\?: UnusedAllowEntryPolicy;/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/resolve-config-types";/,
  );
});

test("CSP allowed origins exports its configuration type", () => {
  const types = readFileSync(join(packageRoot, "csp-allowed-origins-types.d.ts"), "utf8");
  assert.match(types, /export interface CspAllowedOriginsOptions/);
  assert.match(types, /cspFiles: string\[\];/);
  assert.match(types, /selfOrigins\?: string\[\];/);
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./csp-allowed-origins-types";',
    ),
  );
});

test("workspace-relative imports export their package pair configuration types", () => {
  const types = readFileSync(
    join(packageRoot, "cross-package-relative-imports-types.d.ts"),
    "utf8",
  );
  assert.match(
    types,
    /export interface WorkspaceRelativeImportAllowEntry \{\s*\/\*\*.*\*\/\s*importer: string;\s*\/\*\*.*\*\/\s*target: string;/,
  );
  assert.match(
    types,
    /export interface NoCrossPackageRelativeImportsOptions \{\s*allow\?: WorkspaceRelativeImportAllowEntry\[\];/,
  );
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/cross-package-relative-imports-types";/,
  );
});

test("Vitest default excludes exports its configuration type", () => {
  const types = readFileSync(join(packageRoot, "vitest-default-excludes-types.d.ts"), "utf8");
  assert.match(types, /export interface VitestDefaultExcludesOptions/);
  assert.match(types, /config\?: string;/);
  assert.match(types, /files\?: string\[\];/);
  assert.match(types, /requiredGlobs\?: string\[\];/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./vitest-default-excludes-types";',
    ),
  );
});

test("Vitest project property policy exports named configuration types", () => {
  const types = readFileSync(join(packageRoot, "vitest-project-policy-types.d.ts"), "utf8");
  assert.match(types, /export type VitestPropertyLiteral = string \| number \| boolean \| null;/);
  assert.match(types, /export type VitestPropertyLiteralKind = "number" \| "string" \| "boolean";/);
  assert.match(types, /export interface VitestDeclaredProjectPolicy/);
  assert.match(types, /forbiddenValues\?: Record<string, VitestPropertyLiteral\[\]>;/);
  assert.match(types, /forbidLiteralKinds\?: Record<string, VitestPropertyLiteralKind\[\]>;/);
  assert.match(types, /declared\?: VitestDeclaredProjectPolicy;/);
  const declared = types.match(/export interface VitestDeclaredProjectPolicy \{([^}]+)\}/)[1];
  assert.doesNotMatch(declared, /required/);
  assert.match(types, /export interface VitestPropertyParity/);
  assert.match(types, /properties: \[string, string\];/);
  assert.match(types, /compareRoot\?: boolean;/);
  assert.match(types, /export interface VitestProjectPolicyOptions/);
  assert.match(types, /config: string;/);
  assert.match(types, /helpers\?: string\[\];/);
  assert.match(types, /required\?: Record<string, VitestPropertyLiteral>;/);
  assert.match(types, /forbidden\?: string\[\];/);
  assert.match(types, /requiredExcludeGlobs\?: string\[\];/);
  assert.match(types, /parity\?: VitestPropertyParity\[\];/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./vitest-project-policy-types";',
    ),
  );
});

test("PostgreSQL executor options expose additive selection and final exclusions beside replacement", () => {
  for (const file of [
    "postgres-test-state-mutation-types.d.ts",
    "postgres-lock-written-rows-types.d.ts",
    "postgres-bounded-statements-types.d.ts",
    "postgres-query-annotation-types.d.ts",
    "executed-postgres-sql-types.d.ts",
    "postgres-mutable-sql-registry-types.d.ts",
    "postgres-json-relation-ids-types.d.ts",
    "postgres-grouping-not-aggregation-types.d.ts",
    "postgres-duplicate-keyed-lookups-types.d.ts",
    "postgres-uuid-predicate-types.d.ts",
    "postgres-unbounded-identifier-types.d.ts",
  ]) {
    const declarations = readFileSync(join(packageRoot, file), "utf8");
    assert.match(declarations, /executorNames\?: string\[\];/);
    assert.match(declarations, /extraExecutorNames\?: string\[\];/);
    assert.match(declarations, /excludeExecutorNames\?: string\[\];/);
  }
});

require("./api-tests/declarations-lock-written-rows");

test("column CHECK options export named predicates, requirements and exceptions", () => {
  const declarations = readFileSync(
    join(packageRoot, "postgres-column-check-constraints-types.d.ts"),
    "utf8",
  );
  for (const name of [
    "PostgresColumnCheckPredicate",
    "PostgresColumnCheckIntegerType",
    "PostgresColumnCheckRequirement",
    "PostgresColumnCheckAllowEntry",
    "PostgresColumnCheckConstraintsOptions",
  ]) {
    assert.match(declarations, new RegExp(`export (?:type|interface) ${name}\\b`));
  }
  assert.match(declarations, /require: PostgresColumnCheckPredicate;/);
  assert.match(declarations, /types: PostgresColumnCheckIntegerType\[\];/);
  assert.match(declarations, /column: string;/);
  assert.match(declarations, /reason: string;/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-column-check-constraints-types";',
    ),
  );
});

test("materialized-view shape options export their named allowance contracts", () => {
  const types = readFileSync(
    join(packageRoot, "postgres-materialized-view-shape-types.d.ts"),
    "utf8",
  );
  assert.match(types, /export interface PostgresMaterializedViewShapeOptions/);
  assert.doesNotMatch(types, /\bmode\??:/);
  assert.match(types, /schemaCatalogPath: string;/);
  assert.match(types, /minAdditionalIndexes\?: number;/);
  assert.match(types, /windowFunctions\?: string\[\];/);
  assert.match(types, /export interface PostgresMaterializedViewBaseTableAllowance/);
  assert.match(types, /allowBaseTables\?: PostgresMaterializedViewBaseTableAllowance\[\];/);
  assert.match(types, /export interface PostgresMaterializedViewShapeAllowEntry/);
  assert.match(types, /allow\?: PostgresMaterializedViewShapeAllowEntry\[\];/);
  const exports = readFileSync(join(packageRoot, "types.d.ts"), "utf8");
  assert.ok(exports.includes('export type * from "./postgres-materialized-view-shape-types";'));
});

test("catalog views expose indexes with the same named contract as tables", () => {
  const types = readFileSync(join(packageRoot, "postgres-catalog-types.d.ts"), "utf8");
  assert.match(
    types,
    /export interface PostgresCatalogView \{[^}]*indexes\?: Record<string, PostgresCatalogIndex>;/,
  );
});

test("queue wait finding kinds are public without narrowing other queue findings", () => {
  const declarations = readFileSync(join(packageRoot, "queue-report-types.d.ts"), "utf8");
  assert.match(
    declarations,
    /export type QueueWaitFindingKind = "enqueue-and-wait" \| "enqueue-and-wait-declaration";/,
  );
  assert.match(declarations, /export interface QueueCheckFinding \{\s*kind: string;/);
  assert.match(
    readFileSync(join(packageRoot, "report-types.d.ts"), "utf8"),
    /export \* from "\.\/queue-report-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/report-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export \* from "\.\/types";/,
  );
});

test("YAML merge uncertainty is declared in topology reports", () => {
  const topology = readFileSync(join(packageRoot, "workflow-topology-types.d.ts"), "utf8");
  assert.match(topology, /\| "unlocated-yaml-merge"/);
});

test("workflow topology policy exports the reusable workflow lock option", () => {
  const declarations = readFileSync(
    join(packageRoot, "workflow-topology-policy-types.d.ts"),
    "utf8",
  );
  assert.match(declarations, /export interface WorkflowTopologyPolicyOptions \{/);
  assert.match(declarations, /forbidReusableWorkflowConcurrency\?: boolean;/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/workflow-topology-policy-types";/,
  );
});

test("GitHub Actions no-poll-wait-job exports its option type", () => {
  const declarations = readFileSync(
    join(packageRoot, "github-actions-no-poll-wait-job-types.d.ts"),
    "utf8",
  );
  assert.match(declarations, /export interface GithubActionsNoPollWaitJobOptions\b/);
  for (const field of ["pollCommands?: string[];", "ignoreForLoops?: boolean;"]) {
    assert.ok(declarations.includes(field), `missing ${field}`);
  }
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-no-poll-wait-job-types";/,
  );
});

test("GitHub Actions env policy exports named option types", () => {
  const declarations = readFileSync(
    join(packageRoot, "github-actions-env-policy-types.d.ts"),
    "utf8",
  );
  for (const name of ["GithubActionsEnvPolicyValue", "GithubActionsEnvPolicyOptions"]) {
    assert.match(declarations, new RegExp(`export (?:type|interface) ${name}\\b`));
  }
  for (const field of [
    "required?: Record<string, GithubActionsEnvPolicyValue>;",
    "forbidden?: string[];",
    "include?: string[];",
    "unanalyzable?: Unanalyzable;",
  ]) {
    assert.ok(declarations.includes(field), `missing ${field}`);
  }
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-env-policy-types";/,
  );
});

test("GitHub Actions run command policy exports named option types", () => {
  const declarations = readFileSync(
    join(packageRoot, "github-actions-run-command-policy-types.d.ts"),
    "utf8",
  );
  for (const name of [
    "GithubActionsRunCommandArg",
    "GithubActionsRunCommandArgScope",
    "GithubActionsRunCommandForbiddenPattern",
    "GithubActionsRunCommandPolicy",
    "GithubActionsRunCommandPolicyOptions",
  ]) {
    assert.match(declarations, new RegExp(`export (?:type|interface) ${name}\\b`));
  }
  for (const field of [
    "command: string[];",
    "valueFlags?: string[];",
    "whenAnyArgs?: GithubActionsRunCommandArg[];",
    "requireArgs?: GithubActionsRunCommandArg[];",
    "requireAnyArgs?: GithubActionsRunCommandArg[];",
    "forbidArgPatterns?: GithubActionsRunCommandForbiddenPattern[];",
    "commands?: GithubActionsRunCommandPolicy[];",
    "unanalyzable?: Unanalyzable;",
  ]) {
    assert.ok(declarations.includes(field), `missing ${field}`);
  }
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-run-command-policy-types";/,
  );
});

test("changes diff and planner changes have exported public contracts", () => {
  const changes = readFileSync(join(packageRoot, "changes-types.d.ts"), "utf8");
  assert.match(changes, /export interface KeyChanges/);
  assert.match(changes, /export type FixtureAttributePaths = Record<string, string\[\]>/);
  assert.match(changes, /attributePaths\?: FixtureAttributePaths/);
  assert.match(changes, /assertPureJsonLoader\?: boolean;/);
  assert.match(changes, /export interface ChangesDiffOptions/);
  assert.match(changes, /export type ChangesReport/);
  assert.match(changes, /changedFiles\?: string\[\];/);
  assert.match(changes, /export function changesDiff/);
  assert.match(changes, /baseRoot: string/);
  assert.match(changes, /headRoot: string/);
  assert.match(changes, /schemaVersion: 1/);
  assert.match(readFileSync(join(packageRoot, "test-types.d.ts"), "utf8"), /changes\?: string/);
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export \{ changesDiff \} from "\.\/changes-types"/,
  );
});

test("GitHub Actions pr-diff-range and workflow call bindings export named option types", () => {
  const prDiff = readFileSync(join(packageRoot, "github-actions-pr-diff-range-types.d.ts"), "utf8");
  assert.match(prDiff, /export type GithubActionsRevisionPair\b/);
  assert.match(prDiff, /export interface GithubActionsPrDiffRangeOptions \{/);
  for (const field of [
    "requireConsistentRevisionPairs?: boolean;",
    "revisionPairs?: GithubActionsRevisionPair[];",
    "unanalyzable?: Unanalyzable;",
    "baseEnv: string; headEnv: string",
    "baseInput: string; headInput: string",
    "baseFlag: string; headFlag: string",
  ]) {
    assert.ok(prDiff.includes(field), `missing ${field}`);
  }
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-pr-diff-range-types";/,
  );

  const topology = readFileSync(join(packageRoot, "workflow-topology-policy-types.d.ts"), "utf8");
  assert.ok(
    topology.includes("requireExplicitWorkflowCallBindings?: boolean;"),
    "missing requireExplicitWorkflowCallBindings",
  );
  assert.ok(topology.includes("unanalyzable?: Unanalyzable;"), "missing unanalyzable");
});

test("required PR path filters export the workflow scope type", () => {
  const declarations = readFileSync(
    join(packageRoot, "github-actions-no-required-pr-path-filters-types.d.ts"),
    "utf8",
  );
  assert.match(declarations, /export interface GithubActionsNoRequiredPrPathFiltersOptions\b/);
  assert.match(declarations, /workflows: string\[\];/);
  assert.match(declarations, /unanalyzable\?: Unanalyzable;/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/github-actions-no-required-pr-path-filters-types";/,
  );
});
