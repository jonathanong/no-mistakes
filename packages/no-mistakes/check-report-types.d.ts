import type { QueueCheckFinding } from "./queue-report-types";
import type { ReactViolation } from "./react-report-types";

export interface CheckReport {
  react: ReactViolation[];
  queues: QueueCheckFinding[];
  rules: RuleFinding[];
  integration: IntegrationFinding[];
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
