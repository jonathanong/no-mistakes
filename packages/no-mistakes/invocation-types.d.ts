/** Controls shared by every analysis invocation. Durations are in seconds. */
export interface InvocationOptions {
  /** Command execution timeout in whole non-negative seconds. Omit, `0`, or `null` disables it. CLI default remains 30. */
  timeout?: number | null;
  /** Maximum time to wait for the current user's invocation lock (shared across project roots), in whole non-negative seconds. Omit, `0`, or `null` waits indefinitely. CLI default remains 30. */
  lockTimeout?: number | null;
  /** Fail immediately instead of waiting when another invocation holds the lock. */
  failOnLock?: boolean;
  /**
   * Rayon worker count. A positive integer takes precedence over `RAYON_NUM_THREADS`.
   * Omit, `0`, or `null` uses `RAYON_NUM_THREADS`, falling back to CPU count when
   * unset or invalid, matching CLI `--jobs 0`.
   * The first analysis request with valid UTF-8 JSON and invocation controls that
   * acquires the invocation lock initializes the process-wide pool before
   * command-specific options are validated. A later command-validation failure
   * can therefore still select the pool; JSON/control or lock failures cannot.
   * Later calls cannot resize an already initialized pool.
   */
  jobs?: number | null;
  /** `ci` sets unbounded command and lock timeouts. CLI `--profile ci` does the same. */
  profile?: "ci";
  /** Register live, stderr-free phase progress under a unique ID. Use diagnosticsSnapshot({ id }) while the request runs; completion or failure removes the ID. */
  diagnosticsId?: string;
}

export type WithInvocationOptions<T> = T & InvocationOptions;

/** One live span; siblings may overlap and labels form the enclosing phase path. */
export interface ActiveTimingSpan {
  id: number;
  phase: string[];
  startedMs: number;
  elapsedMs: number;
  parallel: boolean;
}

/** Completed durations grouped by label and parallel status within the live request. */
export interface CompletedTimingSpan {
  label: string;
  durationMs: number;
  parallel: boolean;
}

export interface DiagnosticsProgressSnapshot {
  elapsedMs: number;
  startedSpans: number;
  completedSpans: number;
  active: ActiveTimingSpan[];
  completed: CompletedTimingSpan[];
}

/** Poll a live diagnosticsId without waiting for the filesystem invocation lock. Returns null before registration and after completion or failure. */
export function diagnosticsSnapshot(options: {
  id: string;
}): Promise<DiagnosticsProgressSnapshot | null>;
