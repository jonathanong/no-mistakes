import type {
  GithubActionsDownloadIntegrityOptions,
  GithubActionsDownloadIntegritySkipAnnotationScope,
} from "no-mistakes";

export const defaultOptions: GithubActionsDownloadIntegrityOptions = {};
export const stepOptions: GithubActionsDownloadIntegrityOptions = { skipAnnotationScope: "step" };
export const lineOptions: GithubActionsDownloadIntegrityOptions = { skipAnnotationScope: "line" };
export const scope: GithubActionsDownloadIntegritySkipAnnotationScope = "line";

// @ts-expect-error Only step and line are public annotation scopes.
export const invalidScope: GithubActionsDownloadIntegritySkipAnnotationScope = "file";
export const invalidOptions: GithubActionsDownloadIntegrityOptions = {
  // @ts-expect-error A scope is a string union, not a number.
  skipAnnotationScope: 1,
};
