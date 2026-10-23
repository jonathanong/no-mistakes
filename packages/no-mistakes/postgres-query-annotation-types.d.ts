import type { Effort, Unanalyzable } from "./traversal-types";

/** Backward-compatible SQL-specific name for the shared unanalyzable policy. */
export type PostgresUnanalyzableSql = Unanalyzable;

/** Configuration options for the `postgres-require-query-annotation` check rule. */
export interface PostgresRequireQueryAnnotationOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  /** Exact module#export wrappers accepting one string or interpolation-free template literal. */
  sqlLiteralHelpers?: string[];
  /** @deprecated At effort "low" only; ignored there and warns when configured. Still used at effort "max". */
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: PostgresUnanalyzableSql;
}
