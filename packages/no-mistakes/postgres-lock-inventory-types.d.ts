/** Options for `postgresLockInventory()`. */
export interface PostgresLockInventoryOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Create a missing baseline or shrink an existing baseline. Defaults to false. */
  writeBaseline?: boolean;
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

/** Versioned PostgreSQL lock inventory baseline. */
export interface LockInventoryBaseline {
  version: number;
  sites: LockInventoryBaselineSite[];
}

/** Structured result from `postgresLockInventory()`. */
export interface LockInventoryReport {
  sites: LockInventorySite[];
  counts: Record<string, number>;
  unanalyzed: UnanalyzedLockSite[];
  baselineFile: string;
  added: LockInventorySite[];
  removed: LockInventoryBaselineSite[];
  baselineUpdated: boolean;
}
