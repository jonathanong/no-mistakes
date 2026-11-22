/** Options for the opt-in `vitest-default-excludes` check rule. */
export interface VitestDefaultExcludesOptions {
  /** Path to the selected Vitest config relative to the repository root. */
  config: string;
  /** Exact required strings. Defaults to Vitest's node_modules and .git exclusions. */
  requiredGlobs?: string[];
}
