import type {
  PostgresSqlDiagnostic,
  PostgresSqlExpression,
  PostgresSqlSpan,
  PostgresSqlStatement,
} from "./postgres-source-types";
/** Nested facts are procedural source occurrences, never guaranteed execution. */
export interface PostgresSqlProceduralBlock {
  language: string;
  bodyEncoding: PostgresSqlBodyEncoding;
  bodySpan: PostgresSqlSpan;
  statements: PostgresSqlStatement[];
  diagnostics: PostgresSqlDiagnostic[];
  complete: boolean;
}

/** Branch DDL describes source occurrences, never guaranteed execution. */
export interface PostgresSqlConditionalBranch {
  condition: PostgresSqlExpression | null;
  span: PostgresSqlSpan;
  statements: PostgresSqlStatement[];
}

/** Original body source slices retain this enclosing literal encoding. */
export type PostgresSqlBodyEncoding = "dollarQuoted" | "singleQuoted";
