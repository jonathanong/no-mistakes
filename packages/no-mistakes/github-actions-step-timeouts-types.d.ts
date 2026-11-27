import type { Unanalyzable } from "./traversal-types";

export interface GithubActionsStepTimeoutsAllowEntry {
  /** Exact step selector formatted as `path#jobId#stepId` (or 1-based step index). */
  step: string;
  /** Non-empty explanation for the exception. */
  reason: string;
  /** Optional literal timeout cap overriding the rule's cap (1..=360). */
  maxMinutes?: number;
}

export interface GithubActionsStepTimeoutsActionCap {
  /** Concrete action family (for example `"actions/setup-node"` or `"./.github/actions/setup-pnpm"`). */
  uses: string;
  /** Maximum literal `timeout-minutes` allowed for matching workflow steps (1..=360). */
  maxMinutes: number;
}

export interface GithubActionsStepTimeoutsOptions {
  /** Optional repo-relative workflow globs replacing the default workflow YAML globs. */
  include?: string[];
  /** Optional global maximum literal `timeout-minutes` cap (clamped to GitHub's 360-minute limit). */
  maxMinutes?: number;
  /** Reasoned per-step exceptions. */
  allow?: GithubActionsStepTimeoutsAllowEntry[];
  /** Per-action-family step timeout caps (for example setup actions). */
  actionCaps?: GithubActionsStepTimeoutsActionCap[];
  /** When true, require timeouts only on steps matching `actionCaps` (requires non-empty `actionCaps`). */
  onlyConfiguredActions?: boolean;
  /** Policy for dynamic `uses:` steps when `actionCaps` is configured (default: "advise"). */
  unanalyzable?: Unanalyzable;
}
