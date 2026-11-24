export type QueueRetentionKey = "removeOnComplete" | "removeOnFail";

/** Setting this config object enables the unbounded-job-retention check. */
export interface UnboundedJobRetentionConfig {
  /** Defaults to both keys. An empty list checks neither key. */
  keys?: QueueRetentionKey[];
  /** Repository-relative glob patterns. */
  allowPaths?: string[];
}

/** Setting this config object enables the queue-job-state-as-record advisory. */
export interface JobStateAsRecordConfig {
  /** Repository-relative glob patterns for operator tooling and dashboards. */
  allowPaths?: string[];
}
