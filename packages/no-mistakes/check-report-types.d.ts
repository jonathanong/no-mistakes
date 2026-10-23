import type { QueueCheckFinding } from "./queue-report-types";
import type { ReactViolation } from "./react-report-types";
import type { Effort } from "./traversal-types";

/** A low-effort rule that used maximum analysis because a required partner was unavailable. */
export interface AnalysisFallback {
  /** Canonical rule identifier. */
  rule: string;
  /** Required partner check that could not be validated in scope. */
  partner: string;
  requested: Effort;
  effective: Effort;
}

export interface CheckReport {
  react: ReactViolation[];
  queues: QueueCheckFinding[];
  rules: RuleFinding[];
  integration: IntegrationFinding[];
  /** Partner-backed low-effort rules that fell back to maximum analysis; omitted when empty. */
  analysisFallbacks?: AnalysisFallback[];
  /** Present only when requested; declaration evidence, not runtime closure. */
  runnerConfigDeadlines?: RunnerConfigDeadlineEvidence[];
  codebase: UniqueExportFinding[];
  warnings: string[];
  /**
   * SQL-looking member `.query` calls that were not proven to be executors.
   * Omitted when zero. Not a failure and not part of `warnings`.
   */
  unprovenMemberQuerySql?: number;
  advisories: RuleFinding[];
  /** Present only when `includeSqlVerdicts` is requested, including an empty array. */
  sqlVerdicts?: SqlObligationVerdict[];
  /** Present when `includeSuppressed` is requested; empty when no directives matched. */
  suppressed?: SuppressedFinding[];
}

export type SqlVerdictStatus = "violation" | "proven-clear" | "abstain";
export type SqlEvidenceKind = "exact" | "exhaustive-union" | "verified-prefix" | "policy-exemption";
export interface SqlExecutorProfile {
  importSpecifier: string;
  executorNames: string[];
  executorFactoryNames: string[];
  executorTypeNames: string[];
  trustedSqlTags: { module: string; name: string }[];
  sqlLiteralHelpers?: string[];
}
/** Zero-based ordinal among enabled applications of the same rule in config order. */
export interface SqlRuleApplication {
  index: number;
  name?: string;
}
export interface SqlObligationVerdict {
  profile: SqlExecutorProfile;
  rule: string;
  application?: SqlRuleApplication;
  subcheck: string;
  file: string;
  span: [number, number];
  status: SqlVerdictStatus;
  advisory: boolean;
  evidence?: SqlEvidenceKind;
  knownPrefix?: string;
  helperLines?: number[];
  residualReasons?: string[];
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
