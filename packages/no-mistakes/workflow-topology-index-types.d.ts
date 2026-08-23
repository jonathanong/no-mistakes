// The JS-only `createWorkflowTopologyIndex()` query index type, split out
// of `workflow-topology-types.d.ts` to stay under the 200-line file limit.

import type { ArtifactEdge } from "./workflow-topology-artifact-types";
import type {
  WorkflowJobNode,
  WorkflowNode,
  WorkflowRunEdge,
  WorkflowTopology,
} from "./workflow-topology-types";

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
