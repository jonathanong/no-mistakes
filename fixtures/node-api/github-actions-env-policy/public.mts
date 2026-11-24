import type {
  GithubActionsEnvPolicyOptions,
  GithubActionsEnvPolicyValue,
} from "no-mistakes";

const shellInit: GithubActionsEnvPolicyValue = "/dev/null";
const options: GithubActionsEnvPolicyOptions = {
  required: { BASH_ENV: shellInit, CI_RETRIES: 3, CI_DEBUG: false },
  forbidden: ["NODE_OPTIONS"],
  include: [".github/workflows/**/*.yml"],
  unanalyzable: "advise",
};

export { options };
