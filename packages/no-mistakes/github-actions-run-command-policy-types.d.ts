import type { Unanalyzable } from "./resolve-config-types";

export type GithubActionsRunCommandArg = string | string[];

export type GithubActionsRunCommandArgScope = "any" | "positional";

export interface GithubActionsRunCommandForbiddenPattern {
  /** Regular expression matched against arguments in the selected scope. */
  pattern: string;
  /** Arguments to inspect: "any" (default) or "positional" after the subcommand. */
  scope?: GithubActionsRunCommandArgScope;
  /** Optional policy explanation overriding the parent command policy message. */
  message?: string;
}

export interface GithubActionsRunCommandPolicy {
  /** Exact command tokens to match, such as ["gh", "api"] or ["git", "commit"]. */
  command: string[];
  /** Flags known to consume the following argument when written without `=`. */
  valueFlags?: string[];
  /** Optional trigger arguments; when non-empty, apply this policy only if one matches. */
  whenAnyArgs?: GithubActionsRunCommandArg[];
  /** Arguments or adjacent token pairs that must all be present. */
  requireArgs?: GithubActionsRunCommandArg[];
  /** Alternative arguments or adjacent token pairs; at least one must be present. */
  requireAnyArgs?: GithubActionsRunCommandArg[];
  /** Forbidden regular expression patterns checked against arguments. */
  forbidArgPatterns?: GithubActionsRunCommandForbiddenPattern[];
  /** Optional explanation appended to violations from this policy. */
  message?: string;
}

export interface GithubActionsRunCommandPolicyOptions {
  /** Command policies to enforce. Defaults to [] (opt-in, reports nothing). */
  commands?: GithubActionsRunCommandPolicy[];
  /** Optional repo-relative globs narrowing checked workflow and composite-action files. */
  include?: string[];
  /** Policy for unparsable commands (default: "advise"). */
  unanalyzable?: Unanalyzable;
}
