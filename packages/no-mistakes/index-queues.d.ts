import type {
  ProjectOptions,
  QueueCheckFinding,
  QueueCheckOptions,
  QueueCheckReport,
  QueueEdge,
  QueueReport,
  WithInvocationOptions,
  WithInvocationResult,
} from "./types";

export function queues(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<WithInvocationResult<QueueReport>>;
export function queueEdges(options?: WithInvocationOptions<ProjectOptions>): Promise<QueueEdge[]>;
export function queueRelated(options: WithInvocationOptions<ProjectOptions>): Promise<QueueEdge[]>;
export function queueCheck(
  options: WithInvocationOptions<QueueCheckOptions & { includeDiagnostics: true }>,
): Promise<WithInvocationResult<QueueCheckReport>>;
export function queueCheck(
  options?: WithInvocationOptions<QueueCheckOptions & { includeDiagnostics?: false }>,
): Promise<QueueCheckFinding[]>;
export function queueCheck(
  options?: WithInvocationOptions<QueueCheckOptions>,
): Promise<QueueCheckFinding[] | WithInvocationResult<QueueCheckReport>>;
