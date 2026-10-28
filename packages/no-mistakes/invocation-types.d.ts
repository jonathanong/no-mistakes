/** Controls shared by every analysis invocation. timeout/lockTimeout use seconds; invocationTimeoutMs uses milliseconds. */
export interface InvocationOptions {
  /** Command execution timeout in whole non-negative seconds. Omit, `0`, or `null` disables it. CLI default remains 30. */
  timeout?: number | null;
  /** Positive safe-integer native invocation budget in milliseconds, shared by lock acquisition and execution. Never extends timeout or lockTimeout; profile ci disables it. Omit to preserve existing timeout semantics; null is invalid. */
  invocationTimeoutMs?: number;
  /** Maximum time to wait for the current user's invocation lock (shared across project roots), in whole non-negative seconds. Omit, `0`, or `null` waits indefinitely. CLI default remains 30. */
  lockTimeout?: number | null;
  /** Fail immediately instead of waiting when another invocation holds the lock. */
  failOnLock?: boolean;
  /**
   * Rayon worker count. A positive integer takes precedence over `RAYON_NUM_THREADS`.
   * Omit, `0`, or `null` uses `RAYON_NUM_THREADS`, falling back to CPU count when
   * unset, zero, or invalid, matching CLI `--jobs 0`.
   * The first analysis request with valid UTF-8 JSON and invocation controls that
   * acquires the invocation lock initializes the process-wide pool before
   * command-specific options are validated. A later command-validation failure
   * can therefore still select the pool; JSON/control or lock failures cannot.
   * Later calls cannot resize an already initialized pool.
   * The call that installs the pool does not warn. A later call whose resolved
   * count differs keeps that pool, does not fail, and includes `jobsPoolWarning`
   * on the JSON object. Omit, `0`, or `null` does not warn when the resolved
   * count matches the pool. Concurrent `analyzeProject()` calls and same-user CLI
   * runs queue on the per-user invocation lock, and each queued run uses the full pool.
   */
  jobs?: number | null;
  /** `ci` sets unbounded command and lock timeouts. CLI `--profile ci` does the same. */
  profile?: "ci";
  /** Register live, stderr-free phase progress under a unique ID. Use diagnosticsSnapshot({ id }) while the request runs; completion or failure removes the ID. */
  diagnosticsId?: string;
}

export type WithInvocationOptions<T> = T & InvocationOptions;

/** Metadata emitted only on analysis JSON objects, never arrays or scalar results. */
export interface InvocationResultMetadata {
  jobsPoolWarning?: JobsPoolWarning;
}

export type WithInvocationResult<T> = T extends readonly unknown[]
  ? T
  : T extends object
    ? T & InvocationResultMetadata
    : T;

/** Present when a later call's resolved `jobs` cannot resize the process-wide Rayon pool. */
export interface JobsPoolWarning {
  code: "rayon-pool-size";
  message: string;
  /** Worker count resolved from this call's `jobs` and `RAYON_NUM_THREADS`. */
  requestedJobs: number;
  /** Worker count recorded for the process pool. */
  effectiveJobs: number;
}

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

/** Who holds the machine-wide invocation lock, as recorded by the holder. */
export interface LockHolder {
  pid: number;
  /** Working directory of the holder; a pid alone is ambiguous across sandboxes. */
  cwd: string | null;
  /** The holder's command line. Empty for legacy pid-only records. */
  command: string;
  /** Unix epoch milliseconds when the holder acquired the lock; 0 when unknown. */
  startedAtMs: number;
  /** Whole seconds the lock has been held; null when the start time is unknown. */
  ageSeconds: number | null;
}

/** Result of `lockStatus()` / `no-mistakes lock status --format json`. */
export interface LockStatus {
  state: "held" | "free";
  lockFile: string;
  /** Present only while the lock is held. */
  holder: LockHolder | null;
}

/** Stable `kind` values for invocation failures. */
export type InvocationErrorKind = "lock_busy" | "lock_timeout" | "deadline";

/**
 * The `error` object written to stdout under `--format json`, also exposed as
 * `error.details` on rejected Node API calls. CLI exit codes: `lock_busy` and
 * `lock_timeout` exit 75 (EX_TEMPFAIL); `deadline` exits 124.
 */
export interface InvocationErrorDetails {
  kind: InvocationErrorKind;
  /** Phase that was running, for example `invocation.lock` or `check.prepare`. */
  phase: string | null;
  message: string;
  exitCode: 75 | 124;
  /** Present for lock failures when the holder record could be read. */
  holder?: LockHolder;
}

/** Rejection from a Node API call that failed on the invocation lock or deadline. */
export interface InvocationError extends Error {
  code: "NO_MISTAKES_LOCK_BUSY" | "NO_MISTAKES_LOCK_TIMEOUT" | "NO_MISTAKES_DEADLINE";
  details: InvocationErrorDetails;
}
