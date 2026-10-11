import type { QueueCheckFinding } from "./queue-report-types";
import type { ReactViolation } from "./react-report-types";

export interface CheckReport {
  react: ReactViolation[];
  queues: QueueCheckFinding[];
  rules: RuleFinding[];
  integration: IntegrationFinding[];
  /** Present only when requested; declaration evidence, not runtime closure. */
  runnerConfigDeadlines?: RunnerConfigDeadlineEvidence[];
  codebase: UniqueExportFinding[];
  warnings: string[];
  advisories: RuleFinding[];
  /** Present when `includeSuppressed` is requested; empty when no directives matched. */
  suppressed?: SuppressedFinding[];
}

export interface SuppressedFinding {
  domain: "react" | "queues" | "rules" | "filesystem" | "integration" | "codebase" | "advisories";
  rule: string;
  file: string;
  /** File containing the suppression directive. */
  sourceFile: string;
  line?: number;
  reason: string;
  directive: {
    kind: "file" | "line" | "nextLine";
    line: number;
  };
}

export interface RuleFinding {
  rule: string;
  file: string;
  line: number;
  message: string;
  import?: string;
  target?: string;
}

export interface IntegrationFinding {
  framework: string;
  suite: string;
  file: string;
  line: number;
  testName?: string;
  describePath?: string[];
  integration?: string;
  message: string;
}

export interface UniqueExportFinding {
  rule: string;
  file: string;
  line: number;
  exportName: string;
  exportKind: string;
  message: string;
}

export type RunnerDeadlineStatus = "notRequested" | "prepared" | "failed";
export interface RunnerConfigDeadlineEvidence {
  framework: "playwright" | "vitest";
  status: RunnerDeadlineStatus;
  configs: ConfigDeadlineEvidence[];
}
export type ConfigDeadlineEvidence =
  | { status: "prepared"; config: string; projects: ProjectDeadlineEvidence[] }
  | { status: "failed"; config: string; error: string };
export interface ProjectDeadlineEvidence {
  config: string | null;
  workspace: boolean;
  policyName: string | null;
  runnerProjectArg: string | null;
  scope: string | null;
  case: DeclaredDeadlineSlot;
  hook: DeclaredDeadlineSlot;
  fixture: DeclaredDeadlineSlot;
}
export type DeclaredDeadlineSlot =
  | { status: "absent" }
  | { status: "known"; milliseconds: number; provenance: DeadlineProvenance }
  | { status: "unknown"; reason: DeclaredDeadlineUnknownReason; provenance: DeadlineProvenance };
export type DeclaredDeadlineUnknownReason =
  | "expression"
  | "nonFinite"
  | "opaqueSpread"
  | "opaqueTestObject"
  | "computedProperty"
  | "unresolvedExtends"
  | "unresolvedInheritance"
  | "unprovedConfigRoot"
  | "accessor"
  | "unsupportedConfigCall"
  | "unprovedBinding";
export interface DeadlineProvenance {
  /** Request-root-relative slash path; outside-root sources retain their path. */
  path: string;
  /** Half-open source byte offsets, never line numbers; null when unavailable. */
  span: [number, number] | null;
  inheritedThrough: DeadlineInheritanceEvidence[];
}
export interface DeadlineInheritanceEvidence {
  path: string;
  project: string | null;
}
