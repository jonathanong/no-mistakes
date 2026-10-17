import type {
  PostgresSqlDiagnostic,
  PostgresSqlSpan,
  PostgresSqlStatement,
} from "./postgres-source-types";
export type PostgresSqlExecution = "nonExecuting" | "executesForAnalysis" | "unknown";
export type PostgresSqlWrapperKind = "explain" | "prepare" | "functionDeclaration";
/** Ordered child source occurrences inherit the enclosing execution context. */
export interface PostgresSqlWrapper {
  wrapperKind: PostgresSqlWrapperKind;
  execution: PostgresSqlExecution;
  statements: PostgresSqlStatement[];
  span: PostgresSqlSpan | null;
  complete: boolean;
  diagnostics: PostgresSqlDiagnostic[];
}
