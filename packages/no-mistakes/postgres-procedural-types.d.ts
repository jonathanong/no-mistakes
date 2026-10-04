import type {
  PostgresSqlDiagnostic,
  PostgresSqlSpan,
  PostgresSqlStatement,
} from "./postgres-source-types";
/** Nested facts are procedural source occurrences, never guaranteed execution. */
export interface PostgresSqlProceduralBlock {
  language: string;
  bodySpan: PostgresSqlSpan;
  statements: PostgresSqlStatement[];
  diagnostics: PostgresSqlDiagnostic[];
  complete: boolean;
}
