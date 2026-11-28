import type { Unanalyzable } from "./traversal-types";

export interface QueueNoEnqueueInTransactionPublisher {
  module: string;
  export: string;
}

export interface QueueNoEnqueueInTransactionBoundary {
  module: string;
  export: string;
  /** Zero-based argument index of the callback that runs before commit. */
  callbackArgument: number;
}

export interface QueueNoEnqueueInTransactionBegin {
  module: string;
  export: string;
}

/** YAML options for the queue-no-enqueue-in-transaction rule. Configure at least one transaction API. */
export interface QueueNoEnqueueInTransactionOptions {
  transactions?: QueueNoEnqueueInTransactionBoundary[];
  beginTransactions?: QueueNoEnqueueInTransactionBegin[];
  publishers: QueueNoEnqueueInTransactionPublisher[];
  /** Policy for shapes the rule cannot prove (default: "report"). */
  unanalyzable?: Unanalyzable;
}
