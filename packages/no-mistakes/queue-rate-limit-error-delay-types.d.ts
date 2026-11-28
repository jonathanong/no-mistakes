import type { Unanalyzable } from "./traversal-types";

export interface QueueRateLimitErrorDelayTarget {
  /** Import specifier or repository module exporting the rate-limit error or static factory. */
  module: string;
  /** Exported error class or static member path (for example `Worker.RateLimitError` or `GroupRateLimitError`). */
  export: string;
  /** Zero-based argument index that carries the delay duration or options object (default `0` when `instanceDelayProperty` is not set). */
  delayArgument?: number;
  /** Optional property name on the `delayArgument` options object that holds the delay duration. */
  delayProperty?: string;
  /** Optional property name expected on the constructed error instance (for no-arg constructors such as `glide-mq`'s `Worker.RateLimitError`, where the worker reads `error.delayMs`). Mutually exclusive with `delayArgument` and `delayProperty`. */
  instanceDelayProperty?: string;
}

export interface QueueRateLimitErrorDelayOptions {
  /** Configured rate-limit error constructors or static factories to check. */
  targets: QueueRateLimitErrorDelayTarget[];
  /** Policy for calls whose callee or delay arguments cannot be proven statically (default `"report"`). */
  unanalyzable?: Unanalyzable;
}
