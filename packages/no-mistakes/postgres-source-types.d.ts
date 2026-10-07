import type { PostgresSqlFunction } from "./postgres-function-types";
import type { PostgresSqlComment, PostgresSqlAlterIndex } from "./postgres-metadata-types";
export type * from "./postgres-metadata-types";
export type * from "./postgres-function-types";
import type { PostgresSqlInsert } from "./postgres-insert-types";
export type * from "./postgres-insert-types";
import type { PostgresSqlExpressionRoot } from "./postgres-expression-types";
export type * from "./postgres-expression-types";
import type { PostgresSqlQuery } from "./postgres-query-types";
export type * from "./postgres-query-types";
import type { PostgresSqlDrop } from "./postgres-drop-types";
export type * from "./postgres-drop-types";
import type {
  PostgresSqlConditionalBranch,
  PostgresSqlProceduralBlock,
} from "./postgres-procedural-types";
export type * from "./postgres-procedural-types";
/** Pure SQL source input. No repository, database, or filesystem options are needed. */
export interface PostgresSqlSource {
  sql: string;
  fileName?: string | null;
}
/** UTF-8 byte offset, with one-based Unicode scalar line and column. */
export interface PostgresSqlPosition {
  offset: number;
  line: number;
  column: number;
}
/** End-exclusive source span. */
export interface PostgresSqlSpan {
  start: PostgresSqlPosition;
  end: PostgresSqlPosition;
}
export interface PostgresSqlDiagnostic {
  message: string;
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlIdentifier {
  value: string;
  quoted: boolean;
  identity: string;
}
export interface PostgresSqlName {
  parts: PostgresSqlIdentifier[];
  sql: string;
}
export interface PostgresSqlFunctionReference {
  name: PostgresSqlName;
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlExpression {
  sql: string;
  identity: string;
  span: PostgresSqlSpan | null;
  columns: PostgresSqlName[];
  functions: PostgresSqlFunctionReference[];
  root: PostgresSqlExpressionRoot;
}
export interface PostgresSqlType {
  sql: string;
  name: PostgresSqlName | null;
  builtin: string | null;
  modifiers: string[];
  arrayDimensions: (number | null)[];
  fields: PostgresSqlColumn[];
}
export interface PostgresSqlGeneratedColumn {
  expression: PostgresSqlExpression;
  storage: string | null;
}
export interface PostgresSqlIdentityColumn {
  mode: string;
  options: string[];
}
export interface PostgresSqlColumn {
  name: PostgresSqlIdentifier;
  dataType: PostgresSqlType;
  nullable: boolean;
  default: PostgresSqlExpression | null;
  generated: PostgresSqlGeneratedColumn | null;
  identity: PostgresSqlIdentityColumn | null;
  constraints: PostgresSqlConstraint[];
  span: PostgresSqlSpan | null;
}
export type PostgresSqlConstraintKind = "primaryKey" | "unique" | "foreignKey" | "check" | "other";
export interface PostgresSqlConstraint {
  kind: PostgresSqlConstraintKind;
  name: PostgresSqlIdentifier | null;
  columns: PostgresSqlIdentifier[];
  expression: PostgresSqlExpression | null;
  referencedTable: PostgresSqlName | null;
  referencedColumns: PostgresSqlIdentifier[];
  onDelete: string | null;
  onUpdate: string | null;
  characteristics: string | null;
  sql: string;
}
export type PostgresSqlAlterOperation =
  | { kind: "addColumn"; column: PostgresSqlColumn }
  | {
      kind: "alterColumnType";
      column: PostgresSqlIdentifier;
      dataType: PostgresSqlType;
      using: PostgresSqlExpression | null;
    }
  | { kind: "setDefault"; column: PostgresSqlIdentifier; expression: PostgresSqlExpression }
  | { kind: "dropDefault" | "setNotNull" | "dropNotNull"; column: PostgresSqlIdentifier }
  | { kind: "addConstraint"; constraint: PostgresSqlConstraint; notValid: boolean }
  | { kind: "validateConstraint"; name: PostgresSqlIdentifier }
  | { kind: "other"; sql: string };
export interface PostgresSqlIndexKey {
  expression: PostgresSqlExpression;
  ascending: boolean;
  nullsFirst: boolean;
  operatorClass: PostgresSqlName | null;
}
export interface PostgresSqlIndex {
  name: PostgresSqlName | null;
  table: PostgresSqlName;
  method: string;
  unique: boolean;
  nullsDistinct: boolean;
  keys: PostgresSqlIndexKey[];
  include: PostgresSqlIdentifier[];
  predicate: PostgresSqlExpression | null;
  options: string[];
  structuralIdentity: string;
}
export interface PostgresSqlView {
  name: PostgresSqlName;
  columns: PostgresSqlIdentifier[];
  materialized: boolean;
  temporary: boolean;
  orReplace: boolean;
  query: string;
  dependencies: PostgresSqlName[];
  dependenciesComplete: boolean;
  functions: PostgresSqlFunctionReference[];
}
export interface PostgresSqlTriggerTransition {
  kind: string;
  name: PostgresSqlName;
}
export interface PostgresSqlTrigger {
  name: PostgresSqlName;
  table: PostgresSqlName;
  timing: string | null;
  events: string[];
  forEach: string | null;
  condition: PostgresSqlExpression | null;
  function: PostgresSqlName | null;
  arguments: string[];
  constraint: boolean;
  orReplace: boolean;
  referencedTable: PostgresSqlName | null;
  transitions: PostgresSqlTriggerTransition[];
  executionKind: string | null;
  characteristics: string | null;
}
export type PostgresSqlStatementKind =
  | { kind: "comment"; comment: PostgresSqlComment }
  | { kind: "alterIndex"; index: PostgresSqlAlterIndex }
  | { kind: "insert"; insert: PostgresSqlInsert }
  | { kind: "select"; query: PostgresSqlQuery }
  | {
      kind: "createTable";
      table: PostgresSqlName;
      columns: PostgresSqlColumn[];
      constraints: PostgresSqlConstraint[];
      temporary: boolean;
    }
  | { kind: "alterTable"; table: PostgresSqlName; operations: PostgresSqlAlterOperation[] }
  | { kind: "createIndex"; index: PostgresSqlIndex }
  | { kind: "createView"; view: PostgresSqlView }
  | { kind: "createTrigger"; trigger: PostgresSqlTrigger }
  | { kind: "createFunction"; function: PostgresSqlFunction }
  | { kind: "drop"; drop: PostgresSqlDrop }
  | { kind: "doBlock"; block: PostgresSqlProceduralBlock }
  | { kind: "conditional"; branches: PostgresSqlConditionalBranch[] }
  | { kind: "other" };
export type PostgresSqlStatement = PostgresSqlStatementKind & {
  ordinal: number;
  span: PostgresSqlSpan;
  sql: string;
};
/** Versioned facts retain valid neighbors when a statement cannot be parsed. */
export interface PostgresSqlFacts {
  schemaVersion: 1;
  fileName: string | null;
  statements: PostgresSqlStatement[];
  diagnostics: PostgresSqlDiagnostic[];
}
