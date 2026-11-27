import type { Unanalyzable } from "./traversal-types";

export type GithubActionsRevisionPair =
  | { baseEnv: string; headEnv: string }
  | { baseInput: string; headInput: string }
  | { baseFlag: string; headFlag: string };

export interface GithubActionsPrDiffRangeOptions {
  /** Require consistent base/merge or base/head revision pairs in `git diff` commands, tracking `actions/checkout` `with.ref` (default: false). */
  requireConsistentRevisionPairs?: boolean;
  /** Configured env, action/reusable `with:` input, or CLI flag pairs to check for revision consistency (default: []). */
  revisionPairs?: GithubActionsRevisionPair[];
  /** Policy for dynamic revisions, unknown checkout refs, or invalid YAML when consistency checks are enabled (default: "advise"). */
  unanalyzable?: Unanalyzable;
}
