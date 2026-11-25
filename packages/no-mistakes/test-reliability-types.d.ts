/** A repository-relative path glob or literal project name with a reason for the exception. */
export type TestReliabilityAllowlistEntry =
  | { path: string; project?: never; reason: string }
  | { project: string; path?: never; reason: string };

/** A repository-relative module and export name treated as Playwright's `test`. */
export interface PlaywrightFixtureExport {
  module: string;
  export: string;
}

/** Options for the `test-no-retries` check rule. */
export interface TestNoRetriesOptions {
  maxRetries?: number;
  allowlist?: TestReliabilityAllowlistEntry[];
  testCommands?: string[][];
  files?: string[];
  fixtures?: PlaywrightFixtureExport[];
}

/** Options for the `playwright-no-serial` check rule. */
export interface PlaywrightNoSerialOptions {
  allowlist?: TestReliabilityAllowlistEntry[];
  fixtures?: PlaywrightFixtureExport[];
}
