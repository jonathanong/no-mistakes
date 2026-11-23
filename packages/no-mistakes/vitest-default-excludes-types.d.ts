/** Options for the opt-in `vitest-default-excludes` check rule. */
export interface VitestDefaultExcludesOptions {
  /** Path to the selected Vitest config relative to the repository root. */
  config?: string;
  /** Module globs whose `test`-shaped object literals are checked for `test.exclude`. */
  files?: string[];
  /** Exact required strings. Defaults to Vitest's node_modules and .git exclusions. */
  requiredGlobs?: string[];
}
