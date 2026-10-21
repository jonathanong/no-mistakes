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
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: PostgresUnanalyzableSql;
}
