import type { Unanalyzable } from "./traversal-types";

export type GithubActionsEnvPolicyValue = string | number | boolean;

export interface GithubActionsEnvPolicyOptions {
  /** Env names mapped to the only value each may have where it is set. Defaults to {}. */
  required?: Record<string, GithubActionsEnvPolicyValue>;
  /** Env names that must not appear. Defaults to []. */
  forbidden?: string[];
  /** Optional repo-relative globs replacing the default workflow and composite-action files. */
  include?: string[];
  /** Policy for required values written as `${{ }}` expressions (default: "advise"). */
  unanalyzable?: Unanalyzable;
}
