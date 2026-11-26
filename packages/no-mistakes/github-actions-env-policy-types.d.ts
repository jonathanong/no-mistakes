import type { Unanalyzable } from "./traversal-types";

export type GithubActionsEnvPolicyValue = string | number | boolean | null;

export interface GithubActionsEnvPolicyOptions {
  /** Env names mapped to their required value. Null is normalized to the empty string. */
  required?: Record<string, GithubActionsEnvPolicyValue>;
  /** Env names that must not appear. Defaults to []. */
  forbidden?: string[];
  /** Optional repo-relative globs replacing the default workflow and composite-action files. */
  include?: string[];
  /** Policy for required values written as `${{ }}` expressions (default: "advise"). */
  unanalyzable?: Unanalyzable;
}
