export interface GithubActionsNoPollWaitJobOptions {
  /** Command prefixes that poll another run, such as "gh run view". Defaults to [] (reports nothing). */
  pollCommands?: string[];
  /** Skip `for` loops as bounded retries (default: false). */
  ignoreForLoops?: boolean;
}
