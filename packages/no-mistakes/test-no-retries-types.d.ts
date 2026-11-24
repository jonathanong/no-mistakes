/** Allowlist entry for `test-no-retries`. Requires `reason` and at least one of `path` or `project`. */
export interface TestNoRetriesAllowEntry {
  /** Repository-relative file glob exempted from `test-no-retries`. */
  path?: string;
  /** Test runner project name exempted from `test-no-retries`. */
  project?: string;
  /** Required explanation for why retries are temporarily allowed here. */
  reason: string;
}

/** Options for the opt-in `test-no-retries` check rule. */
export interface TestNoRetriesOptions {
  /** Maximum permitted retry count. Defaults to `0`. */
  maxRetries?: number;
  /** Scoped allowlist entries, each with a required `reason`. */
  allow?: TestNoRetriesAllowEntry[];
  /** Optional repository-relative file globs restricting scanned files. */
  files?: string[];
  /** Optional explicit test config file globs in addition to runner configs and standard filenames. */
  configs?: string[];
  /** Additional GitHub Actions `owner/repo` names treated as retry actions. */
  retryActions?: string[];
  /** Additional CI test command prefixes scanned for shell rerun loops and `||` fallbacks. */
  ciTestCommands?: string[];
}
