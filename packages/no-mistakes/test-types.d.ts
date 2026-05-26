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
  /** file#export entrypoints to trace impact from (union of all). */
  entrypoints?: string[];
  environment?: string;
  limitPercent?: number;
  limitFiles?: number;
  globalConfigFallback?: boolean;
}

export interface TestsImpactOptions {
  root?: string;
  config?: string;
  tsconfig?: string;
  /** file#export entrypoints to trace impact from. */
  entrypoints: string[];
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
