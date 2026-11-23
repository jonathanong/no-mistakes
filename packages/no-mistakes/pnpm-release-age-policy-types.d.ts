import type { Effort, Unanalyzable } from "./traversal-types";

/** Permanent exemption for an exact package name or a package-name glob. */
export type PnpmReleaseAgePermanentPackage =
  | { name: string; pattern?: never; reason?: string }
  | { pattern: string; name?: never; reason?: string };

/** Temporary exemptions for exact package versions. */
export interface PnpmReleaseAgeTemporaryGroup {
  selectors: string[];
  reason: string;
  /** Canonical UTC timestamp in YYYY-MM-DDTHH:mm:ssZ format. */
  eligibleForRemovalAt: string;
}

/** Options for `pnpm-release-age-policy`. */
export interface PnpmReleaseAgePolicyOptions {
  permanentPackages?: PnpmReleaseAgePermanentPackage[];
  temporaryGroups?: PnpmReleaseAgeTemporaryGroup[];
  /** Compatibility list of exact package@version selectors. */
  temporarySelectors?: string[];
  scopedPrefixes?: string[];
  workspaceYaml?: string;
  dependabotPath?: string;
  lockfilePath?: string;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
}
