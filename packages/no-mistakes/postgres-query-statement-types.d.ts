import type {
  PostgresSqlDiagnostic,
  PostgresSqlExpression,
  PostgresSqlIdentifier,
  PostgresSqlName,
  PostgresSqlSpan,
} from "./postgres-source-types";
import type { PostgresSqlConflict } from "./postgres-insert-types";
import type { PostgresSqlQueryUnsupported } from "./postgres-query-types";

/** Syntactic modifying bodies in source order, not PostgreSQL execution order.
 * IDs reference the containing PostgresSqlQuery report, including nested CTEs.
 * cteId is null for a modifying query body outside a CTE definition.
 */
export interface PostgresSqlQueryStatementProvenance {
  ordinal: number;
  cteId: number | null;
  queryScopeId: number;
  parentScopeId: number | null;
  /** Exact original source slice. Empty only when span is unavailable/incomplete. */
  sql: string;
  span: PostgresSqlSpan | null;
  returning: PostgresSqlReturningItem[];
  complete: boolean;
  unsupported: PostgresSqlQueryUnsupported[];
}
export type PostgresSqlQueryStatementKind =
  | { kind: "insert"; insert: PostgresSqlCteInsert }
  | { kind: "update"; update: PostgresSqlCteUpdate }
  | { kind: "delete"; delete: PostgresSqlCteDelete }
  | { kind: "merge"; merge: PostgresSqlCteMerge }
  | { kind: "unsupported"; reason: string };
export type PostgresSqlQueryStatement = PostgresSqlQueryStatementProvenance &
  PostgresSqlQueryStatementKind;
export interface PostgresSqlCteInsert {
  table: PostgresSqlName | null;
  alias: PostgresSqlIdentifier | null;
  columns: PostgresSqlName[];
  columnsOmitted: boolean;
  source: PostgresSqlCteInsertSource;
  onConflict: PostgresSqlConflict | null;
  diagnostics: PostgresSqlDiagnostic[];
}
export type PostgresSqlCteInsertSource =
  | { kind: "values"; rows: PostgresSqlExpression[][]; span: PostgresSqlSpan | null }
  | { kind: "select"; queryScopeId: number; span: PostgresSqlSpan | null }
  | { kind: "defaultValues" }
  | { kind: "unsupported"; reason: string };
export interface PostgresSqlCteUpdate {
  targetRelationIds: number[];
  fromRelationIds: number[];
  assignments: PostgresSqlDmlAssignment[];
  predicate: PostgresSqlExpression | null;
}
export interface PostgresSqlCteDelete {
  targetRelationIds: number[];
  usingRelationIds: number[];
  predicate: PostgresSqlExpression | null;
}
export interface PostgresSqlCteMerge {
  targetRelationIds: number[];
  sourceRelationIds: number[];
  predicate: PostgresSqlExpression;
  clauses: PostgresSqlMergeClause[];
}
export interface PostgresSqlDmlAssignment {
  columns: PostgresSqlName[];
  expression: PostgresSqlExpression;
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlMergeClause {
  matchKind: string;
  predicate: PostgresSqlExpression | null;
  span: PostgresSqlSpan | null;
  action: PostgresSqlMergeAction;
}
export type PostgresSqlMergeAction =
  | { kind: "insert"; columns: PostgresSqlName[]; rows: PostgresSqlExpression[][] }
  | { kind: "update"; assignments: PostgresSqlDmlAssignment[] }
  | { kind: "delete" }
  | { kind: "doNothing" }
  | { kind: "unsupported"; reason: string };
export type PostgresSqlReturningItem =
  | { kind: "expression"; expression: PostgresSqlExpression; alias: PostgresSqlIdentifier | null }
  | { kind: "wildcard"; qualifier: PostgresSqlName | null; span: PostgresSqlSpan | null }
  | { kind: "unsupported"; reason: string; span: PostgresSqlSpan | null };
