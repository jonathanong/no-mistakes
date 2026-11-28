import type {
  ProjectOptions,
  QueueCheckFinding,
  QueueCheckOptions,
  QueueCheckReport,
  QueueEdge,
  QueueRelatedOptions,
  QueueReport,
  WithInvocationOptions,
  WithInvocationResult,
} from "./types";

export function queues(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<WithInvocationResult<QueueReport>>;
/** Async queue edges, including Kafka static topic literals. `QueueEdge` names the row. */
export function queueEdges(options?: WithInvocationOptions<ProjectOptions>): Promise<QueueEdge[]>;
export function queueRelated(
  options: WithInvocationOptions<QueueRelatedOptions>,
): Promise<QueueEdge[]>;
export function queueCheck(
  options: WithInvocationOptions<QueueCheckOptions & { includeDiagnostics: true }>,
): Promise<WithInvocationResult<QueueCheckReport>>;
export function queueCheck(
  options?: WithInvocationOptions<QueueCheckOptions & { includeDiagnostics?: false }>,
): Promise<QueueCheckFinding[]>;
export function queueCheck(
  options?: WithInvocationOptions<QueueCheckOptions>,
): Promise<QueueCheckFinding[] | WithInvocationResult<QueueCheckReport>>;
