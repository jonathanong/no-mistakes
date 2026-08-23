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

export interface QueueCheckFinding {
  kind: string;
  file: string;
  line: number;
  queueFile: string | null;
  queueName: string | null;
  job: string | null;
  message: string;
}
