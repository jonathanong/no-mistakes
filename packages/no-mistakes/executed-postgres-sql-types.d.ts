export interface ExecutedPostgresSource {
  fileName: string;
  source: string;
}

export interface ExecutedPostgresTrustedSqlTag {
  module: string;
  name: string;
}

export interface ExecutedPostgresResolvedImport {
  fromFileName: string;
  specifier: string;
  toFileName: string;
}

export interface ExecutedPostgresSqlOptions<Diagnostics extends boolean = false> {
  sources: ExecutedPostgresSource[];
  /** Module whose named imports provide the configured executors. */
  importSpecifier: string;
  /** Defaults to query, read, write when importSpecifier is nonempty. */
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: ExecutedPostgresTrustedSqlTag[];
  /** Opt in to import-aware SQL value/effect projection among supplied sources. */
  resolveImports?: boolean;
  /** Explicit edges for aliases or package imports; relative supplied paths resolve automatically. */
  resolvedImports?: ExecutedPostgresResolvedImport[];
  /** Include omitted executor-type diagnostics without treating them as SQL statements. */
  includeDiagnostics?: Diagnostics;
}

export type ExecutedPostgresSqlKind = "inline" | "immutableLocal" | "composed" | "dynamic";

export interface ExecutedPostgresSql {
  fileName: string;
  /** One-based line of the executor call. */
  line: number;
  annotation: string | null;
  /** Interpolations are represented by sql_placeholder_N markers. */
  sqlText: string | null;
  /** False when full SQL could not be proven, including generated binds. */
  complete: boolean;
  /** True only when all incompleteness is from generated value binds. Parse before classifying reads. */
  onlyValuePlaceholders: boolean;
  kind: ExecutedPostgresSqlKind;
  /** Evaluator blocker; null for complete SQL or proven generated value binds. */
  incompleteReason: string | null;
}

/** Omitted member-call sites grouped by their imported receiver type identity. */
export interface UndeclaredExecutorTypeSummary {
  type: string;
  module: string;
  count: number;
}

export interface ExecutedPostgresSqlReport {
  statements: ExecutedPostgresSql[];
  /** Sorted by type, then module; empty when no undeclared receiver types were found. */
  undeclaredExecutorTypes: UndeclaredExecutorTypeSummary[];
}

/** Extract configured PostgreSQL executor calls from supplied TS/JS source. */
export function extractExecutedPostgresSql(
  options: ExecutedPostgresSqlOptions<true> & { includeDiagnostics: true },
): Promise<ExecutedPostgresSqlReport>;
export function extractExecutedPostgresSql(
  options: ExecutedPostgresSqlOptions,
): Promise<ExecutedPostgresSql[]>;
export function extractExecutedPostgresSql(
  options: ExecutedPostgresSqlOptions<boolean>,
): Promise<ExecutedPostgresSql[] | ExecutedPostgresSqlReport>;
