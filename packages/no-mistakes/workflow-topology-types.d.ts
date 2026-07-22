// Types for `ciTopology()` / `no-mistakes ci topology` and the JS-only
// `createWorkflowTopologyIndex()` query index built from its output.
//
// This mirrors the schema-v1 JSON contract exactly (field names, casing) —
// see `docs/node-api.md` for the stability guarantees (field order and
// array/diagnostic sort order are part of the contract).

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

export type ArtifactValue =
  | { kind: "static"; raw: string; value: string; instanceCount?: number }
  | { kind: "finite"; raw: string; values: string[]; instanceCounts: Record<string, number> }
  | { kind: "dynamic"; raw: string }
  | { kind: "path-derived"; reason: "archive-disabled" };

export type ArtifactActionFlag =
  | { kind: "static"; raw?: string; effective: boolean }
  | { kind: "dynamic"; raw: string };

export interface ArtifactUploadDeclaration {
  kind: "upload";
  name: ArtifactValue;
  archive: ArtifactActionFlag;
  overwrite: ArtifactActionFlag;
}

export type ArtifactDownloadSelector =
  | { kind: "name"; name: ArtifactValue }
  | { kind: "pattern"; pattern: ArtifactValue }
  | { kind: "all" }
  | { kind: "artifact-ids"; artifactIds: ArtifactValue }
  | {
      kind: "unresolved";
      reason: "name-with-artifact-ids";
      name: ArtifactValue;
      artifactIds: ArtifactValue;
    };

export type ArtifactDownloadSource =
  | { kind: "current-run"; repository?: ArtifactValue; runId?: ArtifactValue }
  | { kind: "external"; repository?: ArtifactValue; runId?: ArtifactValue }
  | { kind: "dynamic"; repository?: ArtifactValue; runId?: ArtifactValue };

export interface ArtifactDownloadDeclaration {
  kind: "download";
  selector: ArtifactDownloadSelector;
  source: ArtifactDownloadSource;
}

export type ArtifactDeclaration = ArtifactUploadDeclaration | ArtifactDownloadDeclaration;

export interface ArtifactEdge {
  kind: "artifact";
  from: string;
  to: string;
  name: string;
  producerStep: number;
  consumerStep: number;
  match: "exact" | "pattern" | "all" | "possible";
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
  /** Always absent until the artifact-dataflow resolver lands (a later port wave). */
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

/**
 * A frozen, sorted query index over a {@link WorkflowTopology}, built
 * entirely in JS from `ciTopology()`'s output via
 * {@link createWorkflowTopologyIndex}. Every array-returning method throws
 * `Error("unknown workflow job: <id>")` / `Error("unknown workflow: <path>")`
 * for an id not present in the topology.
 */
export interface WorkflowTopologyIndex {
  readonly workflowsByPath: ReadonlyMap<string, Readonly<WorkflowNode>>;
  readonly jobsById: ReadonlyMap<string, Readonly<WorkflowJobNode>>;
  directUpstreamJobIds(jobId: string): readonly string[];
  transitiveUpstreamJobIds(jobId: string): readonly string[];
  directDownstreamJobIds(jobId: string): readonly string[];
  transitiveDownstreamJobIds(jobId: string): readonly string[];
  directCallerJobIds(workflowPath: string): readonly string[];
  directCallerWorkflowPaths(workflowPath: string): readonly string[];
  transitiveCallerWorkflowPaths(workflowPath: string): readonly string[];
  directCalleeWorkflowPaths(workflowPath: string): readonly string[];
  transitiveCalleeWorkflowPaths(workflowPath: string): readonly string[];
  incomingWorkflowRunEdges(workflowPath: string): readonly Readonly<WorkflowRunEdge>[];
  outgoingWorkflowRunEdges(workflowPath: string): readonly Readonly<WorkflowRunEdge>[];
  directWorkflowRunSourcePaths(workflowPath: string): readonly string[];
  transitiveWorkflowRunSourcePaths(workflowPath: string): readonly string[];
  directWorkflowRunSubscriberPaths(workflowPath: string): readonly string[];
  transitiveWorkflowRunSubscriberPaths(workflowPath: string): readonly string[];
  artifactProducersForConsumerJob(jobId: string): readonly Readonly<ArtifactEdge>[];
  artifactConsumersForProducerJob(jobId: string): readonly Readonly<ArtifactEdge>[];
}
