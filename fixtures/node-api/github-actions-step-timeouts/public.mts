import type {
  GithubActionsStepTimeoutsActionCap,
  GithubActionsStepTimeoutsAllowEntry,
  GithubActionsStepTimeoutsOptions,
} from "no-mistakes";

const cap: GithubActionsStepTimeoutsActionCap = {
  uses: "actions/setup-node",
  maxMinutes: 5,
};
const allow: GithubActionsStepTimeoutsAllowEntry = {
  step: ".github/workflows/ci.yml#build#download",
  reason: "Matrix controls the timeout.",
  maxMinutes: 20,
};
const options: GithubActionsStepTimeoutsOptions = {
  include: [".github/workflows/**/*.yml"],
  maxMinutes: 30,
  allow: [allow],
  actionCaps: [cap, { uses: "./.github/actions/setup-pnpm", maxMinutes: 5 }],
  onlyConfiguredActions: false,
  unanalyzable: "advise",
};

export { options };
