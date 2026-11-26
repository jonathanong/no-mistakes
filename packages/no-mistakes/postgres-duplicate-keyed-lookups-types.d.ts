import type { Unanalyzable } from "./traversal-types";

/** Legacy SQL policy spelling for postgres-no-duplicate-keyed-lookups. */
export type PostgresNoDuplicateKeyedLookupsUnanalyzableSql = "fail" | "ignore" | "advise";

/** Options for the opt-in postgres-no-duplicate-keyed-lookups rule. */
export interface PostgresNoDuplicateKeyedLookupsOptions {
  include?: string[];
  exclude?: string[];
  sqlInclude?: string[];
  importSpecifier?: string;
  /** Replaces query/read/write; mutually exclusive with extraExecutorNames. */
  executorNames?: string[];
  /** Extends query/read/write; mutually exclusive with executorNames. */
  extraExecutorNames?: string[];
  /** Removes names after replacement/defaults and extras are selected. */
  excludeExecutorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  executorOptionProperties?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  /** Shared policy for unanalyzable SQL (`report` fails by default). */
  unanalyzable?: Unanalyzable;
  /** Legacy alias for unanalyzable; canonical `unanalyzable` takes precedence. */
  unanalyzableSql?: PostgresNoDuplicateKeyedLookupsUnanalyzableSql;
}
