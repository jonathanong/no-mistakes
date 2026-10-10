import type {
  PostgresSqlDiagnostic,
  PostgresSqlExpression,
  PostgresSqlSpan,
  PostgresSqlStatement,
} from "./postgres-source-types";
/** Nested facts are procedural source occurrences, never guaranteed execution.
 * Supported nested ALTER TABLE constraints use the same typed operations as
 * top-level statements. Their spans retain original global source coordinates.
 * Inspect diagnostics even when safely attributed statements are present.
 */
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
export type PostgresSqlBodyEncoding = "dollarQuoted" | "singleQuoted" | "escapedString";
/** Concatenated command literals can contain several original string encodings. */
export type PostgresSqlExecuteEncoding = PostgresSqlBodyEncoding | "concatenated";

/** literalSpan and USING expression spans use enclosing source coordinates.
 * Command child spans and diagnostics use decodedSql coordinates, including
 * nested command expression spans.
 * Source occurrences never imply that SQL executes. */
export interface PostgresSqlLiteralExecute {
  /** The whole literal command expression, including concatenation and parentheses. */
  literalSpan: PostgresSqlSpan;
  bodyEncoding: PostgresSqlExecuteEncoding;
  decodedSql: string;
  /** Ordered source expressions, with enclosing source spans and no runtime evaluation.
   * A present span's source bytes equal `sql`, including call parentheses. */
  using: PostgresSqlExpression[];
  statements: PostgresSqlStatement[];
  diagnostics: PostgresSqlDiagnostic[];
  complete: boolean;
}
