/** Revision-aware impact routing over an entry GitHub Actions workflow. */
export interface CiTopologyImpactOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Exact Git revision, e.g. a pull request base SHA. */
  base: string;
  /** Exact Git revision, e.g. GitHub's tested merge SHA. */
  head: string;
  /** Workflow path or basename, restricted to `.github/workflows`. */
  entryWorkflow: string;
}

export interface CiTopologyImpactDiagnostic {
  code: string;
  message: string;
  workflowPath?: string;
  /** Whether the diagnostic maps completely to entry-workflow jobs. */
  scope: "localized" | "global";
  /** Sorted entry job IDs when and only when `scope` is `localized`. */
  rootJobIds?: string[];
}

export interface CiTopologyImpactReport {
  schemaVersion: 1;
  baseRevision: string;
  headRevision: string;
  changedPaths: string[];
  /** Changed recognized workflows plus their reusable-workflow callers. */
  affectedWorkflows: string[];
  /** Directly affected entry jobs plus transitive `needs` prerequisites. */
  affectedRootJobIds: string[];
  diagnostics: CiTopologyImpactDiagnostic[];
  globalFallback: boolean;
}
