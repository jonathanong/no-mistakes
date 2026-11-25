import type {
  GithubActionsRunCommandArg,
  GithubActionsRunCommandArgScope,
  GithubActionsRunCommandForbiddenPattern,
  GithubActionsRunCommandPolicy,
  GithubActionsRunCommandPolicyOptions,
} from "no-mistakes";

const pair: GithubActionsRunCommandArg = ["-c", "core.hooksPath=/dev/null"];
const scope: GithubActionsRunCommandArgScope = "positional";
const forbidden: GithubActionsRunCommandForbiddenPattern = {
  pattern: "\\?",
  scope,
  message: "Use -f/-F fields with --method GET instead of a query string.",
};
const policy: GithubActionsRunCommandPolicy = {
  command: ["gh", "api"],
  valueFlags: ["-f", "-F", "--method", "-X"],
  whenAnyArgs: ["-f", "-F"],
  unlessAnyArgs: ["graphql"],
  requireAnyArgs: ["--method", "-X"],
  forbidArgPatterns: [forbidden],
  message: "Pass --method explicitly with field flags.",
};
const options: GithubActionsRunCommandPolicyOptions = {
  commands: [
    policy,
    {
      command: ["git", "commit"],
      requireArgs: [pair],
    },
  ],
  include: [".github/workflows/**/*.yml"],
  shellFiles: ["scripts/**/*.sh"],
  unanalyzable: "advise",
};

export { options };
