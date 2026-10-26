/** Comment directives selectable by the forbidden-lint-suppressions check. */
export type ForbiddenLintSuppressionDirective =
  | "eslint-disable"
  | "eslint-disable-line"
  | "eslint-disable-next-line"
  | "oxlint-disable"
  | "oxlint-disable-line"
  | "oxlint-disable-next-line"
  | "@ts-ignore"
  | "@ts-expect-error";

/** Options for the opt-in forbidden-lint-suppressions repository check. */
export interface ForbiddenLintSuppressionsOptions {
  /** Required, nonempty repository-relative file globs; intersect the application scope. */
  files: string[];
  /** Exact forbidden lint rule identifiers. Default: []. */
  ruleIds?: string[];
  /** Literal forbidden lint rule identifier prefixes. Default: []. */
  rulePrefixes?: string[];
  /** Defaults to all six eslint-disable/oxlint-disable forms; TypeScript directives are opt-in. */
  directives?: ForbiddenLintSuppressionDirective[];
  /** Also reject selected lint directives with no rule identifiers. Default: false. */
  banBlanketDisable?: boolean;
  /** Custom diagnostic explanation for agents. */
  message?: string;
}
