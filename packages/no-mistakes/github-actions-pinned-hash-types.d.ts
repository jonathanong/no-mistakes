import type { Unanalyzable } from "./traversal-types";

/** Options for the repository github-actions-pinned-hash check rule. */
export interface GithubActionsPinnedHashOptions {
  excludePaths?: string[];
  /** Check service, job container, and docker:// step images. Defaults to false. */
  images?: boolean;
  /** How to report expression-valued images and unreadable workflow files. */
  unanalyzable?: Unanalyzable;
}
