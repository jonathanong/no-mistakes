import type { Unanalyzable } from "./traversal-types";

/** YAML options for workflow-topology-policy, shared by CLI and async check reports. */
export interface WorkflowTopologyPolicyOptions {
  jobInventory?: Record<string, string[]>;
  unlockedWorkflowReasons?: Record<string, string>;
  requiredJobs?: string[];
  forbiddenJobs?: string[];
  forbiddenActivityTypes?: WorkflowForbiddenActivityTypes[];
  forbiddenEvents?: string[];
  forbidSecretsInherit?: boolean;
  /** Defaults to false. Require reusable workflow callers to map every declared secret explicitly and pass every required input. */
  requireExplicitWorkflowCallBindings?: boolean;
  /** Defaults to false. Disallow workflow-level locks on workflow_call workflows. */
  forbidReusableWorkflowConcurrency?: boolean;
  requiredDirectEdges?: [string, string][];
  forbiddenDirectEdges?: [string, string][];
  requiredTransitiveEdges?: [string, string][];
  forbiddenTransitiveEdges?: [string, string][];
  requiredArtifactEdges?: WorkflowArtifactEdgePolicy[];
  exactFanIns?: Record<string, string[]>;
  exactCallerJobs?: Record<string, string[]>;
  stepOrders?: WorkflowStepOrderPolicy[];
  concurrencyPolicy?: Record<string, WorkflowConcurrencyIntent>;
  forbidConcurrencyGroupCollisions?: boolean;
  secretInventory?: WorkflowSecretInventory | null;
  /** Policy for unverified reusable workflow call bindings (default: "advise" when requireExplicitWorkflowCallBindings is true). */
  unanalyzable?: Unanalyzable;
}

export interface WorkflowForbiddenActivityTypes {
  types: string[];
  events?: string[];
}

export interface WorkflowArtifactEdgePolicy {
  from: string;
  to: string;
  name: string;
  match?: string;
}

export interface WorkflowStepSelector {
  id?: string;
  uses?: string;
  name?: string;
}

export interface WorkflowStepOrderPolicy {
  jobId: string;
  steps: WorkflowStepSelector[];
}

export interface WorkflowConcurrencyIntent {
  pending: "coalesce-latest" | "fifo";
  cancellation: "cancel-running" | "retain-running" | "conditional";
  scope: WorkflowConcurrencyScope[];
}

export type WorkflowConcurrencyScope =
  | "pull-request"
  | "ref"
  | "sha"
  | "run"
  | "event"
  | "input-resource"
  | "fixed-resource";

export interface WorkflowSecretInventory {
  repository?: string[];
  environments?: Record<string, string[]>;
  builtInExemptions?: string[];
  reportUnused?: boolean;
}
