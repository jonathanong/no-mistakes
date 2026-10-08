import type {
  PostgresSqlDiagnostic,
  PostgresSqlExpression,
  PostgresSqlIdentifier,
  PostgresSqlName,
  PostgresSqlSpan,
} from "./postgres-source-types";
import type { PostgresSqlQuery } from "./postgres-query-types";
/** Syntactic facts only: complete does not establish replay safety. */
export interface PostgresSqlInsert {
  table: PostgresSqlName | null;
  alias: PostgresSqlIdentifier | null;
  columns: PostgresSqlName[];
  columnsOmitted: boolean;
  source: PostgresSqlInsertSource;
  onConflict: PostgresSqlConflict | null;
  span: PostgresSqlSpan | null;
  /** Syntax completeness is independent of provenance and does not imply replay safety. */
  complete: boolean;
  diagnostics: PostgresSqlDiagnostic[];
}
export type PostgresSqlInsertSource =
  | { kind: "values"; rows: PostgresSqlExpression[][]; span: PostgresSqlSpan | null }
  | { kind: "select"; query: PostgresSqlQuery; span: PostgresSqlSpan | null }
  | { kind: "defaultValues" }
  | { kind: "unsupported"; reason: string };
export interface PostgresSqlConflict {
  target: PostgresSqlConflictTarget;
  predicate: PostgresSqlExpression | null;
  action: PostgresSqlConflictAction;
  span: PostgresSqlSpan | null;
}
export type PostgresSqlConflictTarget =
  | { kind: "omitted" }
  | { kind: "columns"; columns: PostgresSqlIdentifier[] }
  | { kind: "constraint"; name: PostgresSqlName }
  | { kind: "expressions"; expressions: PostgresSqlExpression[] };
export type PostgresSqlConflictAction =
  | { kind: "doNothing" }
  | {
      kind: "doUpdate";
      assignments: PostgresSqlInsertAssignment[];
      predicate: PostgresSqlExpression | null;
    };
/** Ordered index expressions retain source spans; catalog resolution is not implied. */
export interface PostgresSqlAssignmentTarget {
  base: PostgresSqlExpression;
  subscripts: PostgresSqlExpression[];
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlInsertAssignment {
  columns: PostgresSqlName[];
  target?: PostgresSqlAssignmentTarget;
  expression: PostgresSqlExpression;
  /** Conservative syntactic lineage; derived and unresolved do not imply missing syntax. */
  provenance: PostgresSqlInsertProvenance;
  span: PostgresSqlSpan | null;
  /** Whether the assignment syntax is fully represented, independently of provenance. */
  complete: boolean;
}
export type PostgresSqlInsertProvenance =
  | "targetColumn"
  | "excludedColumn"
  | "literal"
  | "placeholder"
  | "derived"
  | "unresolved";
