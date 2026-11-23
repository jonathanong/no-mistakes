export interface DependabotCooldownPolicyOptions {
  /** Required nonnegative integer floor for default and explicit SemVer cooldowns. */
  minimumDays: number;
  /** Require an explicit nonnegative integer open-pull-requests-limit. Defaults to false. */
  requireOpenPullRequestsLimit?: boolean;
  /** Whole-value globs for cooldown.exclude entries. An empty array allows no exclusions. */
  allowedCooldownExcludes?: string[];
  /** Exact package-ecosystem names exempt from all checks in this rule. */
  exemptEcosystems?: string[];
}
