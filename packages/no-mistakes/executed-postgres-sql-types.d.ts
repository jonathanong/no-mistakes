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

export interface ExecutedPostgresSqlOptions {
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
}

/** Extract configured PostgreSQL executor calls from supplied TS/JS source. */
export function extractExecutedPostgresSql(
  options: ExecutedPostgresSqlOptions,
): Promise<ExecutedPostgresSql[]>;
