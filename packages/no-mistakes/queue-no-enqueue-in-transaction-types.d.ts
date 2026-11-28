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

/** YAML options for the queue-no-enqueue-in-transaction rule. */
export interface QueueNoEnqueueInTransactionOptions {
  transactions: QueueNoEnqueueInTransactionBoundary[];
  publishers: QueueNoEnqueueInTransactionPublisher[];
  /** Policy for shapes the rule cannot prove (default: "report"). */
  unanalyzable?: Unanalyzable;
}
