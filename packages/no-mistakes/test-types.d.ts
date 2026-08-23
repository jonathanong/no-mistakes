import type { SymbolEntrypoint } from "./traversal-types";

/** Runner accepted by `testsPlan`, `testsTargets`, and `TestExecutionTarget`. */
export type TestPlanFramework =
  | "vitest"
  | "playwright"
  | "dotnet"
  | "swift"
  | "python"
  | "go"
  | "cargo"
  | "rails"
  | "php"
  | "java"
  | "kotlin"
  | "elixir"
  | "dart"
  | "jest";

interface TestsPlanOptionsBase {
  framework?: TestPlanFramework;
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  base?: string;
  head?: string;
  /** Git diff refspec, e.g. "origin/main...HEAD". Sugar for base/head. */
  fromGitDiff?: string;
  changedFiles?: string[];
  changedFilesFile?: string;
  /** Inline unified diff content to extract changed files from. */
  diff?: string;
  /** Entrypoints to trace impact from: strings may use file#export, or pass { file, symbol }. */
  entrypoints?: Array<string | SymbolEntrypoint>;
  /** Enables symbol fields in entrypoints and symbol-node traversal. */
  includeSymbols?: boolean;
  environment?: string;
  limitPercent?: number;
  limitFiles?: number;
  globalConfigFallback?: boolean;
  /** Include the markdown PR comment as `comment` on the returned plan. */
  includeComment?: boolean;
  /** Keep only selected tests whose relative path matches one of these globs. */
  includeGlob?: string[];
}

/**
 * Options for `testsPlan()`. Direct-owner plans intentionally bypass normal
 * test-plan policy, so their incompatible options are rejected at compile time.
 */
export type TestsPlanOptions =
  | (Omit<
      TestsPlanOptionsBase,
      "framework" | "limitPercent" | "limitFiles" | "globalConfigFallback"
    > & {
      /** Select changed framework-owned tests plus tests one reverse graph edge away. */
      directTestOwner: true;
      framework: TestPlanFramework;
      /** Direct-owner selection is bounded to changed files; use testsImpact for explicit entrypoints. */
      entrypoints?: never;
      limitPercent?: never;
      limitFiles?: never;
      globalConfigFallback?: never;
    })
  | (TestsPlanOptionsBase & {
      /** Set to true only with an explicit framework and without policy overrides. */
      directTestOwner?: false;
    });

export interface TestsImpactOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  /** Entrypoints to trace impact from: strings may use file#export, or pass { file, symbol }. */
  entrypoints: Array<string | SymbolEntrypoint>;
  /** Enables symbol fields in entrypoints and symbol-node traversal. */
  includeSymbols?: boolean;
}

export interface TestsTargetsOptions {
  framework: TestPlanFramework;
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  files: string[];
}

export interface TestPlan {
  /** Complete deterministic changed-file inventory, relative to the request root. */
  changedFiles: string[];
  selectedTests: SelectedTest[];
  groups?: TestPlanGroup[];
  warnings: TestPlanWarning[];
  fallbackTriggered: boolean;
  fallbackReason?: string | null;
  executionTargets?: GroupedExecutionTarget[];
  comment?: string | null;
}

export interface GroupedExecutionTarget {
  runner: TestPlanFramework;
  config?: string | null;
  project?: string | null;
  /** Path-prefix display name, such as a Swift package root. */
  name?: string;
  baseCommand: string[];
  runnerArgs: string[];
  testFiles: string[];
}

export interface SelectedTest {
  testFile: string;
  confidence: "low" | "medium" | "high";
  reasons: ImpactReason[];
  targets?: TestExecutionTarget[];
}

export interface TestExecutionTarget {
  runner: TestPlanFramework;
  config?: string | null;
  /** True when config is a Vitest workspace/project-array source rendered with --workspace. */
  workspace?: boolean;
  project?: string | null;
  /** Path-prefix display name, such as a Swift package root. */
  name?: string;
  baseCommand: string[];
  runnerArgs: string[];
}

export interface ImpactReason {
  changedFile: string;
  path: string[];
  via: string[];
  /** When present, aligns index-for-index with `via`. */
  viaDetails?: Array<ImpactEdgeDetail | null>;
}

export type ImpactEdgeDetail = ResourceImpactEdgeDetail | VitestSetupImpactEdgeDetail;

export interface ResourceImpactEdgeDetail {
  type: "resource";
  consumerFile: string;
  callSites: ResourceCallSite[];
}

export interface VitestSetupImpactEdgeDetail {
  type: "vitest-setup";
  field: "setupFiles" | "globalSetup";
}

export interface ResourceCallSite {
  callKind: ResourceCallKind;
  line: number;
}

/** Static runtime filesystem API that created a resource dependency edge. */
export type ResourceCallKind =
  | "read-file"
  | "read-file-sync"
  | "read-directory"
  | "read-directory-sync"
  | "glob"
  | "glob-sync";

export interface TestPlanGroup {
  type: string;
  selected: string[];
  remaining: number;
  limit?: number | null;
}

export interface TestPlanWarning {
  type: string;
  message: string;
  file: string;
  line?: number;
}

export interface TestsTargetsReport {
  framework: TestPlanFramework;
  tests: TestTargetRow[];
  warnings: TestTargetWarning[];
}

export interface TestTargetRow {
  testFile: string;
  targets: TestExecutionTarget[];
}

export interface TestTargetWarning {
  type: string;
  message: string;
  file: string;
}

export interface TestsWhyOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  test: string;
  changed?: string;
  plan?: string;
  planJson?: SavedTestPlan | string;
}

export interface WhyStep {
  node: string;
  via?: string | null;
  detail?: ImpactEdgeDetail | null;
}

/** A current or pre-`changedFiles` plan accepted by saved-plan document APIs. */
export type SavedTestPlan = TestPlan;

export interface TestsPlanDocumentOptions {
  plan?: string;
  planJson?: SavedTestPlan | string;
}

export interface TestGraph {
  nodes: Array<{ name: string; type: "changed" | "test" | "intermediate" }>;
  edges: Array<{ from: string; to: string; via: string; detail?: ImpactEdgeDetail }>;
}

export interface LockfileDiffOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  base: string;
  head?: string;
  lockfile?: string;
}

export interface LockfileDiffEntry {
  lockfile: string;
  manager: "npm" | "pnpm" | "yarn" | "bun";
  added: string[];
  removed: string[];
  changed: string[];
}
