import type { TraverseOptions, TsConfigDiagnostic } from "./traversal-types";

export type TraverseProjection = "graph" | "paths";

export type DependencyBoundOptions = {
  /** Pre-traversal include globs for the initial candidate inventory. */
  candidateInclude?: string[];
  /** Pre-traversal exclude globs. Reachable imports still escape this set. */
  candidateExclude?: string[];
};

export type TraverseGraphOptions = TraverseOptions &
  DependencyBoundOptions & {
    projection?: "graph";
  };

export type TraversePathsOptions = TraverseOptions &
  DependencyBoundOptions & {
    projection: "paths";
  };

export interface ImportClosureResult {
  files: string[];
  diagnostics: TsConfigDiagnostic[];
}
