import type { ImpactReason, TestPlan } from "./test-types";

/** Caller-recorded immutable identities; audit compares but does not verify the digests. */
export interface TestAuditProvenance {
  /** Lowercase 40- or 64-digit Git revision. */
  checkoutRevision: string;
  /** SHA-256 of the complete source/config snapshot, including dirty changes. */
  sourceDigest: string;
  /** SHA-256 of the runner/project/config/instrumentation scope. */
  scopeDigest: string;
}

export interface TestAuditSymbol {
  file: string;
  /** Exact exported/callable identity agreed upon by the producer and adapter. */
  symbol: string;
}

export interface TestAuditPlanArtifact {
  schemaVersion: 1;
  provenance: TestAuditProvenance;
  plan: TestPlan;
  /** Explicit root-relative scope; must match plan.changedFiles when nonempty. */
  changedFiles: string[];
  /** Refines matching for these files to exact symbols rather than any file execution. */
  changedSymbols: TestAuditSymbol[];
}

export interface TestAuditObservation {
  testFile: string;
  executedFiles: string[];
  executedSymbols: TestAuditSymbol[];
  /** Complete file AND symbol tracing for the declared scope; false makes negative evidence unknown. */
  traceComplete: boolean;
}

export interface TestAuditObservationsArtifact {
  schemaVersion: 1;
  provenance: TestAuditProvenance;
  granularity: "per-test-file";
  suite: "full";
  /** Declares the full suite finished; does not assert instrumentation completeness. */
  complete: true;
  /** One merged observation per test file; aggregate suite coverage is unsupported. */
  tests: TestAuditObservation[];
}

export interface TestsAuditPlanFileInput {
  plan: string;
  planJson?: never;
}

export interface TestsAuditPlanInlineInput {
  plan?: never;
  planJson: TestAuditPlanArtifact | string;
}

export interface TestsAuditObservationsFileInput {
  observations: string;
  observationsJson?: never;
}

export interface TestsAuditObservationsInlineInput {
  observations?: never;
  observationsJson: TestAuditObservationsArtifact | string;
}

export type TestsAuditPlanInput = TestsAuditPlanFileInput | TestsAuditPlanInlineInput;
export type TestsAuditObservationsInput =
  | TestsAuditObservationsFileInput
  | TestsAuditObservationsInlineInput;
/** Requires exactly one path or inline artifact for each input. */
export type TestsAuditOptions = TestsAuditPlanInput & TestsAuditObservationsInput;

export interface TestAuditExecutionEvidence {
  testFile: string;
  matchedFiles: string[];
  matchedSymbols: TestAuditSymbol[];
}

export interface TestAuditSelectionEvidence {
  testFile: string;
  /** Original static selection reasons for investigating excess selection. */
  reasons: ImpactReason[];
}

export interface TestAuditReport {
  provenance: TestAuditProvenance;
  observedTestFiles: number;
  selectedTestFiles: number;
  selectedObservedTests: TestAuditExecutionEvidence[];
  missedObservedTests: TestAuditExecutionEvidence[];
  selectedWithoutObservedExecution: TestAuditSelectionEvidence[];
  selectedWithoutObservations: string[];
  selectedWithIncompleteTraces: string[];
  /** Always explains the limits of observed execution and caller-recorded provenance. */
  limitations: string[];
}
