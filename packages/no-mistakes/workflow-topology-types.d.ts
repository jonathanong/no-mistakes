// Types for `ciTopology()` / `no-mistakes ci topology` and the JS-only
// `createWorkflowTopologyIndex()` query index built from its output.
//
// This mirrors the schema-v1 JSON contract exactly (field names, casing) —
// see `docs/node-api.md` for the stability guarantees (field order and
// array/diagnostic sort order are part of the contract).

import type { ArtifactDeclaration, ArtifactEdge } from "./workflow-topology-artifact-types";

export * from "./workflow-topology-artifact-types";
export * from "./workflow-topology-index-types";

export interface CiTopologyOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /**
   * Restrict output to these workflow(s) (basename or a path inside
   * `.github/workflows`) plus their transitive local reusable-workflow
   * callees. Omit or pass `[]` for every workflow.
   */
  workflows?: string[];
}

export type JsonScalar = boolean | number | string;
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export type ConcurrencyValue = boolean | string;

export interface WorkflowConcurrency {
  raw: { group: string; cancelInProgress?: ConcurrencyValue; queue?: string };
  effective: { group: string; cancelInProgress: ConcurrencyValue; queue: string };
}

export interface WorkflowTrigger {
  event: string;
  config?: JsonValue;
}

export interface WorkflowCallInput {
  type?: "boolean" | "number" | "string";
  required: boolean;
  default?: JsonScalar;
  description?: string;
}

export interface WorkflowCallSecret {
  required: boolean;
  description?: string;
}

export interface WorkflowCallOutput {
  value?: string;
  description?: string;
}

export interface WorkflowCallContract {
  inputs: Record<string, WorkflowCallInput>;
  secrets: Record<string, WorkflowCallSecret>;
  outputs: Record<string, WorkflowCallOutput>;
}

export type WorkflowNode = {
  id: string;
  path: string;
  name: string;
  triggers: WorkflowTrigger[];
  jobIds: string[];
  concurrency?: WorkflowConcurrency;
} & (
  | { callable: true; workflowCall: WorkflowCallContract }
  | { callable: false; workflowCall?: undefined }
);

export interface WorkflowStep {
  index: number;
  kind: "action" | "run" | "other";
  id?: string;
  name?: string;
  condition?: string;
  uses?: string;
  /** Present when this step is an `actions/{upload,download}-artifact` action. */
  artifact?: ArtifactDeclaration;
}

export interface WorkflowJobNode {
  id: string;
  workflowId: string;
  key: string;
  kind: "job" | "matrix-template";
  name?: string;
  condition?: string;
  matrix?: JsonValue;
  concurrency?: WorkflowConcurrency;
  steps: WorkflowStep[];
}

export interface NeedsEdge {
  kind: "needs";
  from: string;
  to: string;
}

export interface WorkflowCallEdge {
  kind: "calls";
  from: string;
  target: string;
  local: boolean;
  bindings: {
    inputs: Record<string, JsonScalar>;
    secrets: { mode: "inherit" } | { mode: "explicit"; values: Record<string, JsonScalar> };
  };
  /** Present only for local (`./`) calls. */
  to?: string;
}

export interface WorkflowRunEdge {
  kind: "workflow-run";
  from: string;
  to: string;
  types?: string[];
  branches?: string[];
  branchesIgnore?: string[];
}

export type WorkflowTopologyEdge = NeedsEdge | WorkflowCallEdge | WorkflowRunEdge | ArtifactEdge;

export type WorkflowTopologyDiagnosticCode =
  | "malformed-workflow"
  | "missing-needs-dependency"
  | "job-dependency-cycle"
  | "duplicate-step-id"
  | "unknown-step-reference"
  | "non-prior-step-reference"
  | "duplicate-workflow-name"
  | "missing-local-workflow"
  | "non-callable-workflow"
  | "workflow-call-cycle"
  | "missing-workflow-run-source"
  | "ambiguous-workflow-run-source"
  | "workflow-run-cycle"
  | "workflow-run-chain-limit"
  | "unknown-workflow-filter"
  | "invalid-workflow-filter"
  | "missing-workflow-call-input"
  | "unknown-workflow-call-input"
  | "workflow-call-input-type-mismatch"
  | "missing-workflow-call-secret"
  | "unknown-workflow-call-secret"
  | "unknown-workflow-call-output"
  | "missing-artifact-producer"
  | "ambiguous-artifact-producer"
  | "artifact-resolution-limit";

export interface WorkflowTopologyDiagnostic {
  severity: "error";
  code: WorkflowTopologyDiagnosticCode;
  message: string;
  workflowPath: string;
  jobId?: string;
  stepIndex?: number;
  callJobId?: string;
  calleeWorkflowPath?: string;
}

export interface WorkflowTopology {
  schemaVersion: 1;
  workflows: WorkflowNode[];
  jobs: WorkflowJobNode[];
  edges: WorkflowTopologyEdge[];
  diagnostics: WorkflowTopologyDiagnostic[];
}
