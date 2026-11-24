import type { ProjectOptions } from "./traversal-types";

export interface QueueReport {
  producers: QueueProducer[];
  workers: QueueWorker[];
  jobs: QueueJobNode[];
  edges: QueueEdge[];
  diagnostics: QueueDiagnostic[];
  check: QueueCheckFinding[];
}

export interface QueueJobNode {
  queueFile: string;
  queueName: string;
  job: string;
}

export interface QueueProducer {
  file: string;
  line: number;
  queueFile: string | null;
  queueName: string | null;
  job: string | null;
  rawJob: string | null;
  library: string | null;
}

export interface QueueWorker {
  file: string;
  line: number;
  processorFile: string | null;
  queueFile: string | null;
  queueName: string | null;
  jobs: string[];
  wildcard: boolean;
  library: string | null;
}

export type QueueEdgeKind = "queue-enqueue" | "queue-worker";

export interface QueueEdge {
  from: string;
  to: string;
  kind: QueueEdgeKind;
}

export type QueueDiagnosticSeverity = "warning" | "error";

export interface QueueDiagnostic {
  severity: QueueDiagnosticSeverity;
  file: string;
  line: number;
  message: string;
}

export type QueueWaitFindingKind = "enqueue-and-wait" | "enqueue-and-wait-declaration";

export type QueueRateLimitFindingKind = "shared-queue-rate-limit" | "worker-rate-limit-per-process";

export type QueuePolicyFindingKind = "unbounded-job-retention" | "queue-job-state-as-record";

export type QueueFindingSeverity = "advisory";

export interface QueueCheckFinding {
  kind: string;
  file: string;
  line: number;
  queueFile: string | null;
  queueName: string | null;
  job: string | null;
  message: string;
  severity?: QueueFindingSeverity;
}

export interface QueueCheckOptions extends ProjectOptions {
  /** Return `{ findings, diagnostics }` instead of a bare findings array. */
  includeDiagnostics?: boolean;
}

export interface QueueCheckReport {
  findings: QueueCheckFinding[];
  diagnostics: QueueDiagnostic[];
}
