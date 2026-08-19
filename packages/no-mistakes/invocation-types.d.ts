/** Controls shared by every analysis invocation. Durations are in seconds. */
export interface InvocationOptions {
  /** Command execution timeout. Defaults to 30; 0 or null disables it. */
  timeout?: number | null;
  /** Maximum time to wait for the machine-wide lock. Defaults to 30; 0 or null waits indefinitely. */
  lockTimeout?: number | null;
  /** Fail immediately instead of waiting when another invocation holds the lock. */
  failOnLock?: boolean;
  /**
   * Rayon worker count. Omit to leave the process pool unchanged.
   * `0` uses the CPU count, matching CLI `--jobs 0`.
   */
  jobs?: number | null;
}

export type WithInvocationOptions<T> = T & InvocationOptions;
