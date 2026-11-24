/** Allowlist entry for `playwright-no-serial`. Requires `reason` and at least one of `path` or `project`. */
export interface PlaywrightNoSerialAllowEntry {
  /** Repository-relative file glob exempted from `playwright-no-serial`. */
  path?: string;
  /** Playwright project name exempted from `playwright-no-serial`. */
  project?: string;
  /** Required explanation for why serial execution is temporarily allowed here. */
  reason: string;
}

/** Options for the opt-in `playwright-no-serial` check rule. */
export interface PlaywrightNoSerialOptions {
  /** Scoped allowlist entries, each with a required `reason`. */
  allow?: PlaywrightNoSerialAllowEntry[];
  /** Optional repository-relative file globs restricting scanned files. */
  files?: string[];
  /** Optional explicit Playwright config file globs in addition to `tests.playwright.configs`. */
  configs?: string[];
}
