export interface LockfileDiffOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  base: string;
  head?: string;
  lockfile?: string;
}

export interface LockfileDiffReportOptions extends LockfileDiffOptions {
  /** Return entries and non-fatal binary lockfile warnings together. */
  includeWarnings: true;
}

export interface LockfileDiffWarning {
  code: "binary-lockfile";
  lockfile: string;
  message: string;
}

export interface LockfileDiffReport {
  entries: LockfileDiffEntry[];
  warnings: LockfileDiffWarning[];
}

export interface LockfileDiffEntry {
  lockfile: string;
  manager: "npm" | "pnpm" | "yarn" | "bun";
  added: string[];
  removed: string[];
  changed: string[];
}
