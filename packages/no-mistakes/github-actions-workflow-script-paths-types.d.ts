import type { Unanalyzable } from "./traversal-types";

export type GithubActionsPathFilterEvent = "push" | "pull_request" | "pull_request_target";

export interface GithubActionsWorkflowScriptPathsOptions {
  /** Repository-relative literal script directories to enforce inside workflow path filters. Defaults to [] (opt-in). */
  scriptDirs?: string[];
  /** Optional repo-relative globs narrowing direct workflow files under .github/workflows/. */
  include?: string[];
  /** Optional subset of path-filterable events to check. Defaults to all three. */
  events?: GithubActionsPathFilterEvent[];
  /** Policy for unprovable triggers, scripts, or local composite actions (default: "advise"). */
  unanalyzable?: Unanalyzable;
}
