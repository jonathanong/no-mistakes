import type { Effort, Unanalyzable } from "./traversal-types";

export interface GithubActionsActionTimeoutPairOptions {
  include?: string[];
  uses?: string[];
  stepTimeoutMinutes?: number;
  nestedInput?: string;
  nestedTimeoutSeconds?: number;
  /** Require the nested input in seconds to be strictly below the step's timeout. Defaults to false. */
  nestedBelowStepTimeout?: boolean;
  forbidNestedInComposite?: boolean;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
}
