import type { Unanalyzable } from "./traversal-types";

export interface GithubActionsNoRequiredPrPathFiltersOptions {
  /** Exact repository-relative workflow paths. An empty list selects no workflows. */
  workflows?: string[];
  unanalyzable?: Unanalyzable;
}
