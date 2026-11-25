import type { Unanalyzable } from "./invocation-types";

export interface QueueRateLimitErrorDelayTarget {
  /** Import specifier or repository module exporting the rate-limit error or static factory. */
  module: string;
  /** Exported error class or static member path (for example `RateLimitError` or `Worker.RateLimitError`). */
  export: string;
  /** Zero-based argument index that carries the delay duration or options object (default `0`). */
  delayArgument?: number;
  /** Optional property name on the `delayArgument` options object that holds the delay duration. */
  delayProperty?: string;
}

export interface QueueRateLimitErrorDelayOptions {
  /** Configured rate-limit error constructors or static factories to check. */
  targets: QueueRateLimitErrorDelayTarget[];
  /** Policy for calls whose callee or delay arguments cannot be proven statically (default `"report"`). */
  unanalyzable?: Unanalyzable;
}
