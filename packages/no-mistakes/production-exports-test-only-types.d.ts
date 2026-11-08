import type { Unanalyzable } from "./traversal-types";

/** A platform that consumes named exports from one exact root-relative file. */
export interface PlatformConsumedExports {
  /** Exact file path relative to the analysis root. */
  file: string;
  /** Exact exported names consumed by the platform; use `default` explicitly. */
  exports: string[];
}

/** Explicit scope for the opt-in production-exports-only-used-by-tests rule. */
export interface ProductionExportsOnlyUsedByTestsOptions {
  /** Required production directories or file paths, relative to the root. */
  productionRoots: string[];
  /** Explicit test-file globs. At least one test glob or helper root is required. */
  testGlobs?: string[];
  /** Explicit test-only helper directories or files; no names are inferred. */
  testHelperRoots?: string[];
  /** Count same-file value/type uses of exported bindings; default: false. */
  ignoreExportsUsedInFile?: boolean;
  /** Exact platform-consumed exports; sibling exports remain independently analyzed. */
  platformConsumedExports?: PlatformConsumedExports[];
  /** Uncertain symbol consumers follow the shared policy; default: report. */
  unanalyzable?: Unanalyzable;
}
