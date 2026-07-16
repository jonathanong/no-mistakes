/** Controls shared by every analysis invocation. Durations are in seconds. */
export interface InvocationOptions {
  /** Command execution timeout. Defaults to 30; 0 or null disables it. */
  timeout?: number | null;
  /** Maximum time to wait for the machine-wide lock. Defaults to 30; 0 or null waits indefinitely. */
  lockTimeout?: number | null;
  /** Fail immediately instead of waiting when another invocation holds the lock. */
  failOnLock?: boolean;
}

export type WithInvocationOptions<T> = T & InvocationOptions;
