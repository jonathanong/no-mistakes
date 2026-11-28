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

/** `queue-enqueue` ends at `queueFile#job`; `queue-worker` starts there. A Kafka job is `<cluster>:<topic>` when the project cluster is set, and the bare topic otherwise. */
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

export type QueueCheckSeverity = "error" | "advisory";

export interface QueueCheckFinding {
  kind: string;
  file: string;
  line: number;
  queueFile: string | null;
  queueName: string | null;
  job: string | null;
  message: string;
  severity?: QueueCheckSeverity;
}

export interface QueueRelatedOptions extends ProjectOptions {
  files: [string, ...string[]];
}

export interface QueueCheckOptions extends ProjectOptions {
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Return `{ findings, diagnostics }` instead of a bare findings array. */
  includeDiagnostics?: boolean;
}

export interface QueueCheckReport {
  findings: QueueCheckFinding[];
  diagnostics: QueueDiagnostic[];
}
