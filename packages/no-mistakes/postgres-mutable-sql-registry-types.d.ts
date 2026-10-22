import type { Effort, Unanalyzable } from "./traversal-types";

/** Configuration for the `postgres-no-mutable-sql-registry` check rule. */
export interface PostgresNoMutableSqlRegistryOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
}
