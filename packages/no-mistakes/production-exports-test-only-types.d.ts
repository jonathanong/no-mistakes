import type { Unanalyzable } from "./traversal-types";

/** Explicit scope for the opt-in production-exports-only-used-by-tests rule. */
export interface ProductionExportsOnlyUsedByTestsOptions {
  /** Required production directories or file paths, relative to the root. */
  productionRoots: string[];
  /** Explicit test-file globs. At least one test glob or helper root is required. */
  testGlobs?: string[];
  /** Explicit test-only helper directories or files; no names are inferred. */
  testHelperRoots?: string[];
  /** Uncertain symbol consumers follow the shared policy; default: report. */
  unanalyzable?: Unanalyzable;
}
