import type { Effort, Unanalyzable } from "./traversal-types";

export type GithubActionsDownloadIntegritySkipAnnotationScope = "step" | "line";

export interface GithubActionsDownloadIntegrityOptions {
  trustedHosts?: string[];
  downloadHelpers?: string[];
  shellSourceInclude?: string[];
  strictSequencing?: boolean;
  skipAnnotationScope?: GithubActionsDownloadIntegritySkipAnnotationScope;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
}
