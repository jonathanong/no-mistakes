import type { DependencyResult, SymbolsResult, TestPlan } from "./types";

/** Input for writing a private, CLI-compatible planning-impact artifact set. */
export interface WritePlanningImpactArtifactsOptions {
  /** Repository root passed to the prepared `analyzeProject()` request. */
  root: string;
  /** Newline-separated, repository-relative changed-file paths inside `outputDirectory`. */
  changedFilesManifest: string;
  /** Existing private directory with exactly mode `0700` that receives the artifacts. */
  outputDirectory: string;
  /** Omit the import/workspace relationship filter for structural reports. */
  broad?: boolean;
}

/** Structured result mirrored by `dependencies.json`, `dependents.json`, `symbols.json`, and `plan.json`. */
export interface PlanningImpactArtifacts {
  outputDirectory: string;
  dependencies: DependencyResult;
  dependents: DependencyResult;
  symbols: SymbolsResult;
  plan: TestPlan;
}
