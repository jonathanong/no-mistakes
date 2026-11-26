import type { Unanalyzable } from "./traversal-types";

export interface GithubActionsNoRequiredPrPathFiltersOptions {
  /** Required non-empty list of exact repository-relative workflow paths directly under `.github/workflows`. */
  workflows: string[];
  unanalyzable?: Unanalyzable;
}
