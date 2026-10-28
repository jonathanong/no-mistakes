/** Root selectors shared with `forbidden-calls`; Playwright roots are not accepted. */
export type TestNoLiveRepositoryScanRoot =
  | { vitest: true | string | string[] }
  | { glob: string | string[] }
  | { file: string }
  | { module: string }
  | { function: { file: string; symbol: string } };

/** An import whose call enumerates a directory tree, in addition to the built-in set. */
export interface TestNoLiveRepositoryScanEnumerator {
  /** Module specifier; a `node:` prefix is ignored. */
  module: string;
  /** Export name; omit to match every export of `module`. */
  export?: string;
}

/** Options for `test-no-live-repository-scan`. */
export interface TestNoLiveRepositoryScanOptions {
  /** Traversal roots. Default: every Vitest project. */
  roots?: TestNoLiveRepositoryScanRoot[];
  /** Glob or globs removed from the root set. */
  excludeRoots?: string | string[];
  /** Path segments marking bounded fixture input; replaces `["fixtures", "__fixtures__"]`. */
  fixtureDirs?: string[];
  /** Extra enumerating imports. */
  enumerators?: TestNoLiveRepositoryScanEnumerator[];
  /** How unresolved scan roots are reported; default: `"advise"`. They are never omitted. */
  unresolvedRoot?: "advise" | "report";
}
