import type { TraverseOptions, TsConfigDiagnostic } from "./traversal-types";

export type TraverseGraphOptions = Omit<TraverseOptions, "projection"> & {
  projection?: "graph";
};

export type TraversePathsOptions = Omit<TraverseOptions, "projection"> & {
  projection: "paths";
};

export interface ImportClosureResult {
  files: string[];
  diagnostics: TsConfigDiagnostic[];
}
