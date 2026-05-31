import type { SymbolEntrypoint } from "./traversal-types";

export interface TestsPlanOptions {
  framework?: "vitest" | "playwright";
  root?: string;
  config?: string;
  tsconfig?: string;
  base?: string;
  head?: string;
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

export interface TestsImpactOptions {
  root?: string;
  config?: string;
  tsconfig?: string;
  /** Entrypoints to trace impact from: strings may use file#export, or pass { file, symbol }. */
  entrypoints: Array<string | SymbolEntrypoint>;
  /** Enables symbol fields in entrypoints and symbol-node traversal. */
  includeSymbols?: boolean;
}

export interface TestPlan {
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
  runner: "vitest" | "playwright";
  config?: string | null;
  project?: string | null;
  base_command: string[];
  runner_args: string[];
}

export interface ImpactReason {
  changed_file: string;
  path: string[];
  via: string[];
}

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
}

export interface TestsWhyOptions {
  root?: string;
  config?: string;
  tsconfig?: string;
  test: string;
  changed?: string;
  plan?: string;
}

export interface WhyStep {
  node: string;
  via?: string | null;
}

export interface TestsPlanDocumentOptions {
  plan?: string;
  planJson?: TestPlan | string;
}

export interface TestGraph {
  nodes: Array<{ name: string; type: "changed" | "test" | "intermediate" }>;
  edges: Array<{ from: string; to: string; via: string }>;
}
