import type { Effort, Unanalyzable } from "./traversal-types";

/** Configuration for the `postgres-no-mutable-sql-registry` check rule. */
export interface PostgresNoMutableSqlRegistryOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  /** Replaces default executor names; mutually exclusive with extraExecutorNames. */
  executorNames?: string[];
  /** Extends query/read/write; mutually exclusive with executorNames. */
  extraExecutorNames?: string[];
  /** Removes names after replacement/defaults and extras are selected. */
  excludeExecutorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
