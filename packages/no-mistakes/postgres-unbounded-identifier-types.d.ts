import type { Effort, Unanalyzable } from "./traversal-types";

/** Configuration for the `postgres-no-unbounded-sql-identifier` check rule. */
export interface PostgresNoUnboundedSqlIdentifierOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
