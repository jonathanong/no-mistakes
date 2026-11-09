/** Stable shape code carried in the advisory finding's `target` field. */
export type PostgresGroupingNotAggregationCode =
  | "all-of-candidate-pool"
  | "distinct-over-unique-key"
  | "distinct-over-union"
  | "distinct-join-filter"
  | "aggregate-existence-test";

/** Options for the always-advisory postgres-grouping-not-aggregation rule. */
export interface PostgresGroupingNotAggregationOptions {
  /** Catalog of primary keys and unique indexes. Missing or empty disables all shapes. */
  schemaCatalogPath?: string;
  include?: string[];
  exclude?: string[];
  sqlInclude?: string[];
  /** Internal resolved-scope flag; defaults to false. */
  sqlIncludeLocked?: boolean;
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
  sqlLiteralHelpers?: string[];
}
