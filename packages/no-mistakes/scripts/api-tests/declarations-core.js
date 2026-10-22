const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("analyzeProject declarations mirror report-specific runtime requirements", () => {
  const traversalDeclarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  const analyzeProjectDeclarations = readFileSync(
    join(packageRoot, "analyze-project-types.d.ts"),
    "utf8",
  );
  assert.match(
    traversalDeclarations,
    /export type SymbolsSignatureImpactOptions = SymbolsOptions & \{/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /options: WithInvocationOptions<SymbolsOptions>,\n\): Promise<WithInvocationResult<SymbolsResult \| SignatureImpactResult>>;/,
  );
  assert.match(traversalDeclarations, /mode: "signature-impact";\n  symbol: string;/);
  assert.match(
    traversalDeclarations,
    /export interface CheckOptions extends ProjectOptions \{[\s\S]*?effort\?: Effort;[\s\S]*?includeSuppressed\?: boolean;/,
  );
  assert.match(
    traversalDeclarations,
    /export interface ProjectOptions \{[\s\S]*?effort\?: Effort;/,
  );
  assert.match(
    readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8"),
    /export interface AnalysisFallback \{[\s\S]*?rule: string;[\s\S]*?partner: string;[\s\S]*?requested: Effort;[\s\S]*?effective: Effort;/,
  );
  assert.doesNotMatch(
    traversalDeclarations,
    /export interface ProjectOptions \{[^}]*includeSuppressed/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /check\(\s*options\?: WithInvocationOptions<CheckOptions>,?\s*\): Promise<WithInvocationResult<CheckReport>>;/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "symbols"; id\?: string } & \(SymbolsListOptions \| SymbolsSignatureImpactOptions\)/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "importUsages"; id\?: string } & Omit<ImportUsagesOptions, "root">/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type BatchedTraverseOptions = TraverseOptions & Pick<ProjectOptions, "config">/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "dependencies"; id\?: string } & BatchedDependencyTraverseOptions/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "resolveCheckDependencies";\n      id\?: string;\n      dependencyReportIds: string\[\];\n      \/\*\* Project root\. Defaults to the current working directory\. \*\/\n      root\?: string;\n      \/\*\* Path to tsconfig\.json for alias resolution\. Searched upward if omitted\. \*\/\n      tsconfig\?: string;\n      \/\*\* Path to the no-mistakes config file \(e\.g\. \.no-mistakes\.yml\)\. Auto-discovered in root if omitted\. \*\/\n      config\?: string;/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "dependents" \| "related"; id\?: string } & BatchedTraverseOptions/,
  );
  assert.doesNotMatch(
    analyzeProjectDeclarations,
    /Omit<\s*TraverseOptions,\s*"root" \| "tsconfig"/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /options\?: WithInvocationOptions<ImportUsagesOptions>,\n\): Promise<WithInvocationResult<ImportUsagesResult>>;/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type BatchedQueueRelatedOptions = BatchedProjectOptions & \{ files: string\[\] \}/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type BatchedServerRouteRelatedOptions = BatchedProjectOptions &\n  \(\{ files: string\[\] \} \| \{ roots: string\[\] \}\)/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "playwrightRelated"; id\?: string } & Omit<PlaywrightRelatedOptions,/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type BatchedReactUsagesOptions = Pick<[\s\S]*?"root" \| "tsconfig" \| "config" \| "targets" \| "include" \| "effort"[\s\S]*?Required<Pick<ProjectOptions, "target">>/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "reactAnalyze" \| "reactCheck"; id\?: string \} & Pick<[\s\S]*?"targets" \| "depth" \| "assertNoFetch" \| "effort"/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "reactUsages"; id\?: string } & BatchedReactUsagesOptions/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type BatchedCheckOptions = Pick<[\s\S]*?"root" \| "tsconfig" \| "config" \| "includeSuppressed" \| "effort"[\s\S]*?>/,
  );
  assert.doesNotMatch(
    analyzeProjectDeclarations,
    /export type BatchedCheckOptions = Pick<[\s\S]*?"include" \| "includeSuppressed"/,
  );
  assert.match(analyzeProjectDeclarations, /type: "check"; id\?: string } & BatchedCheckOptions/);
  assert.match(
    analyzeProjectDeclarations,
    /type: "importers"; id\?: string } & BatchedQueryFileOptions<ImportersOptions>/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "fetches"; id\?: string } & BatchedRootConfigOptions<FetchesOptions>/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "testsPlan"; id\?: string } & BatchedRootTsConfigOptions<TestsPlanOptions>/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "lockfileDiff"; id\?: string } & Omit<LockfileDiffOptions, "root">/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "validateMermaidMarkdown"; id\?: string } & MermaidValidationOptions/,
  );
  assert.match(
    readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8"),
    /domain:[\s\S]*\| "advisories";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8"),
    /unprovenMemberQuerySql\?: number;/,
  );
  assert.match(
    readFileSync(join(packageRoot, "check-report-types.d.ts"), "utf8"),
    /analysisFallbacks\?: AnalysisFallback\[\];/,
  );
  assert.match(
    readFileSync(join(packageRoot, "report-types.d.ts"), "utf8"),
    /export \* from "\.\/check-report-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/report-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/analyze-project-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export \* from "\.\/types";/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type AnalyzeProjectReportType = AnalyzeProjectReportRequest\["type"\];/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export type AnalyzeProjectReportResult = unknown \| EffectsReport \| RscCallersReport;/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /export interface AnalyzeProjectReport \{\n  id\?: string;\n  type: AnalyzeProjectReportType;\n  result: AnalyzeProjectReportResult;\n\}/,
  );
  assert.match(analyzeProjectDeclarations, /reports: AnalyzeProjectReport\[\];/);
  assert.doesNotMatch(analyzeProjectDeclarations, /type: AnalyzeProjectReportRequest\[/);
});

test("query annotation configuration exposes the unknown SQL policy", () => {
  const types = readFileSync(join(packageRoot, "postgres-query-annotation-types.d.ts"), "utf8");
  assert.match(types, /export type PostgresUnanalyzableSql = Unanalyzable;/);
  assert.match(types, /import type \{ Effort, Unanalyzable \} from "\.\/traversal-types";/);
  assert.match(types, /export interface PostgresRequireQueryAnnotationOptions/);
  assert.match(types, /effort\?: Effort;/);
  assert.match(
    types,
    /@deprecated At effort "low" only; ignored there and warns when configured\. Still used at effort "max"\./,
  );
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(types, /unanalyzableSql\?: PostgresUnanalyzableSql/);
  const traversal = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  assert.match(traversal, /export type Effort = "low" \| "max";/);
  assert.match(traversal, /export type Unanalyzable = "report" \| "advise" \| "ignore";/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/postgres-query-annotation-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export \* from "\.\/types";/,
  );
});

test("unbounded SQL identifier options expose shared analysis policy", () => {
  const types = readFileSync(join(packageRoot, "postgres-unbounded-identifier-types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresNoUnboundedSqlIdentifierOptions/);
  for (const field of [
    "include",
    "exclude",
    "executorNames",
    "executorFactoryNames",
    "executorTypeNames",
  ]) {
    assert.ok(types.includes(`${field}?: string[];`));
  }
  assert.match(types, /importSpecifier\?: string;/);
  assert.match(types, /effort\?: Effort;/);
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(types, /unanalyzableSql\?: Unanalyzable;/);
  assert.match(types, /trustedSqlTags\?: \{ module: string; name: string \}\[\];/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-unbounded-identifier-types";',
    ),
  );
});
