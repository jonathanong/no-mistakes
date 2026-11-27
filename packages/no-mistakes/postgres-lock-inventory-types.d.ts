/** Options for `postgresLockInventory()`. */
export interface PostgresLockInventoryOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Create a missing baseline or shrink an existing baseline. Defaults to false. */
  writeBaseline?: boolean;
}

/** Configuration for the opt-in `postgres-lock-inventory` check rule. */
export interface PostgresLockInventoryRuleOptions {
  include?: string[];
  exclude?: string[];
  sqlInclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  extraExecutorNames?: string[];
  excludeExecutorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  executorOptionProperties?: string[];
  executorOptions?: { name: string }[];
  trustedSqlTags?: { module: string; name: string }[];
  /** Repository-relative path to the JSON baseline. Defaults to `.no-mistakes/postgres-locks.json`. */
  baselineFile?: string;
  /** Alias for `baselineFile`. */
  baseline?: string;
  /**
   * Opt-in mode that baselines per-(file, reason) counts of unanalyzed inputs alongside resolved
   * lock-site keys. Defaults to `false`.
   */
  baselineUnanalyzed?: boolean;
  helpers?: string[];
}

/** One configured PostgreSQL lock site. */
export interface LockInventorySite {
  key: string;
  file: string;
  line: number;
  category: string;
}

/** A configured SQL site whose lock behavior could not be analyzed. */
export interface UnanalyzedLockSite {
  file: string;
  line: number;
  reason: string;
}

/** Stable site identity stored in the baseline. */
export interface LockInventoryBaselineSite {
  key: string;
  file: string;
  category: string;
}

/** Per-(file, reason) count of unanalyzed inputs stored in the baseline when `baselineUnanalyzed` is enabled. */
export interface LockInventoryBaselineUnanalyzed {
  file: string;
  reason: string;
  count: number;
}

/** Change in unanalyzed input count for one `(file, reason)` pair relative to the baseline. */
export interface LockInventoryUnanalyzedDelta {
  file: string;
  reason: string;
  baselineCount: number;
  currentCount: number;
}

/** Versioned PostgreSQL lock inventory baseline. */
export interface LockInventoryBaseline {
  version: number;
  sites: LockInventoryBaselineSite[];
  unanalyzed?: LockInventoryBaselineUnanalyzed[];
}

/** Structured result from `postgresLockInventory()`. */
export interface LockInventoryReport {
  sites: LockInventorySite[];
  counts: Record<string, number>;
  unanalyzed: UnanalyzedLockSite[];
  unanalyzedByReason: Record<string, number>;
  unanalyzedByDirectory: Record<string, number>;
  baselineFile: string;
  baselineUnanalyzed: boolean;
  added: LockInventorySite[];
  removed: LockInventoryBaselineSite[];
  unanalyzedAdded: LockInventoryUnanalyzedDelta[];
  unanalyzedRemoved: LockInventoryUnanalyzedDelta[];
  baselineUpdated: boolean;
}
