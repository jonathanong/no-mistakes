import type {
  GithubActionsPathFilterEvent,
  GithubActionsWorkflowScriptPathsOptions,
} from "no-mistakes";

const event: GithubActionsPathFilterEvent = "pull_request";
const options: GithubActionsWorkflowScriptPathsOptions = {
  scriptDirs: ["scripts", "tools/ci"],
  include: [".github/workflows/**/*.yml"],
  events: [event, "push"],
  unanalyzable: "advise",
};

export { options };
