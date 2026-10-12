export interface ExecutedPostgresSource {
  fileName: string;
  source: string;
}

export interface ExecutedPostgresSqlOptions {
  sources: ExecutedPostgresSource[];
  /** Module whose named imports provide the configured executors. */
  importSpecifier: string;
  /** Defaults to query, read, write when importSpecifier is nonempty. */
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
}

export interface ExecutedPostgresSql {
  fileName: string;
  /** One-based line of the executor call. */
  line: number;
  annotation: string | null;
  /** Interpolations are represented by sql_placeholder_N markers. */
  sqlText: string | null;
  /** False when full SQL could not be proven, including interpolation. */
  complete: boolean;
  kind: "inline" | "immutableLocal" | "composed" | "dynamic";
}
