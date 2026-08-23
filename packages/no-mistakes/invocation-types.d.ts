/** Controls shared by every analysis invocation. Durations are in seconds. */
export interface InvocationOptions {
  /** Command execution timeout in whole non-negative seconds. Omit, `0`, or `null` disables it. CLI default remains 30. */
  timeout?: number | null;
  /** Maximum time to wait for the machine-wide lock, in whole non-negative seconds. Omit, `0`, or `null` waits indefinitely. CLI default remains 30. */
  lockTimeout?: number | null;
  /** Fail immediately instead of waiting when another invocation holds the lock. */
  failOnLock?: boolean;
  /**
   * Rayon worker count. Omit to leave the process pool unchanged.
   * `0` uses the CPU count, matching CLI `--jobs 0`.
   */
  jobs?: number | null;
  /** `ci` sets unbounded command and lock timeouts. CLI `--profile ci` does the same. */
  profile?: "ci";
}

export type WithInvocationOptions<T> = T & InvocationOptions;
