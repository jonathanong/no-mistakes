import type { DependabotCooldownPolicyOptions } from "no-mistakes";

export const cooldown: DependabotCooldownPolicyOptions = {
  minimumDays: 7,
  requireOpenPullRequestsLimit: true,
  allowedCooldownExcludes: ["@example/*"],
  exemptEcosystems: ["docker"],
};
