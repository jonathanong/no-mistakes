import type { SymbolEntrypoint } from "./traversal-types";

type TestPlanFramework =
  | "vitest"
  | "playwright"
  | "dotnet"
  | "swift"
  | "python"
  | "go"
  | "cargo"
  | "rails"
  | "php";

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
  changed_files: string[];
  selected_tests: SelectedTest[];
  groups?: TestPlanGroup[];
  warnings: TestPlanWarning[];
  fallback_triggered: boolean;
  fallback_reason?: string | null;
}

export interface SelectedTest {
  test_file: string;
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
  base_command: string[];
  runner_args: string[];
}

export interface ImpactReason {
  changed_file: string;
  path: string[];
  via: string[];
  /** When present, aligns index-for-index with `via`. */
  via_details?: Array<ImpactEdgeDetail | null>;
}

export type ImpactEdgeDetail = ResourceImpactEdgeDetail | VitestSetupImpactEdgeDetail;

export interface ResourceImpactEdgeDetail {
  type: "resource";
  consumer_file: string;
  call_sites: ResourceCallSite[];
}

export interface VitestSetupImpactEdgeDetail {
  type: "vitest-setup";
  field: "setupFiles" | "globalSetup";
}

export interface ResourceCallSite {
  call_kind: ResourceCallKind;
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
}

export interface WhyStep {
  node: string;
  via?: string | null;
  detail?: ImpactEdgeDetail | null;
}

/** A current or pre-`changed_files` plan accepted by saved-plan document APIs. */
export type SavedTestPlan = Omit<TestPlan, "changed_files"> & {
  changed_files?: string[];
};

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
