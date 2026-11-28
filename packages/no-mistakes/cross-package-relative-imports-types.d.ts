import type { Unanalyzable } from "./traversal-types";

/** One allowed relative crossing between declared workspace package names. */
export interface WorkspaceRelativeImportAllowEntry {
  /** Importing package name glob. Use "*" for all importers. */
  importer: string;
  /** Target package name glob. */
  target: string;
}

/** Rule options for no-cross-package-relative-imports. */
export interface NoCrossPackageRelativeImportsOptions {
  allow?: WorkspaceRelativeImportAllowEntry[];
  /** Computed or unresolved imports follow the shared policy; default: advise. */
  unanalyzable?: Unanalyzable;
}
