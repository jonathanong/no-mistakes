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

/** An analyzer command a test must not run against the live working directory. */
export interface TestNoLiveRepositoryScanAnalyzer {
  /** Executable name. A path matches by its basename, including `node_modules/.bin`. */
  command: string;
  /** Leading arguments that identify the analyzer. Extra arguments may follow. */
  args?: string[];
}

/** A shipped asset tree that production code reads at runtime. */
export interface TestNoLiveRepositoryScanRuntimeAsset {
  /** Repository-relative directory, for example `assets/migrations`. No `..`, no leading `/`. */
  root: string;
  /** Why the tree is shipped input rather than a source or policy tree. Required. */
  reason: string;
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
  /**
   * Shipped asset roots. A literal read at or under one is listed as an advisory with
   * target `runtime asset` instead of failing; other trees are still reported.
   */
  runtimeAssets?: TestNoLiveRepositoryScanRuntimeAsset[];
  /**
   * Analyzer commands tests must not run against the live working directory.
   * Default: none. No command name is built in.
   */
  analyzers?: TestNoLiveRepositoryScanAnalyzer[];
}
