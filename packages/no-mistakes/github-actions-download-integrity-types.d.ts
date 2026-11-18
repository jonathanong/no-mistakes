import type { Effort, Unanalyzable } from "./traversal-types";

export interface GithubActionsDownloadIntegrityOptions {
  trustedHosts?: string[];
  downloadHelpers?: string[];
  shellSourceInclude?: string[];
  strictSequencing?: boolean;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
}
