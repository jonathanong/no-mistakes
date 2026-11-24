import type { GithubActionsNoPollWaitJobOptions } from "no-mistakes";

const options: GithubActionsNoPollWaitJobOptions = {
  pollCommands: ["gh run view", "gh run list"],
  ignoreForLoops: true,
};

export { options };
