import type { Effort, Unanalyzable } from "./traversal-types";

/** Opt-in GitHub Actions authored cache-key templates and setup deadline caps. */
export interface GitHubActionsCacheSetupPolicyOptions {
  include?: string[];
  caches?: GitHubActionsCacheKeyShape[];
  setup?: GitHubActionsSetupDeadline[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
}

export interface GitHubActionsCacheKeyShape {
  /** Action family without an @ref. */
  uses: string;
  path: string;
  /** Rust regular expression matching the entire authored key. */
  keyPattern: string;
}

export interface GitHubActionsSetupDeadline {
  /** Action family without an @ref. */
  uses: string;
  /** Maximum literal setup-step timeout in minutes, from 1 through 360. */
  maxMinutes: number;
}
