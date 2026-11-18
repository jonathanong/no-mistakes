import type { Unanalyzable } from "./traversal-types";

/** Stable shape code carried in the advisory finding's `target` field. */
export type PostgresGroupingNotAggregationCode =
  | "all-of-candidate-pool"
  | "distinct-over-unique-key"
  | "distinct-over-union"
  | "distinct-join-filter"
  | "aggregate-existence-test";

/** Legacy SQL policy: fail emits advice for this always-advisory rule. Defaults to advise. */
export type PostgresGroupingNotAggregationUnanalyzableSql = "fail" | "ignore" | "advise";

/** Options for the always-advisory postgres-grouping-not-aggregation rule. */
export interface PostgresGroupingNotAggregationOptions {
  /** Catalog of primary keys and unique indexes. Missing or empty disables all shapes. */
  schemaCatalogPath?: string;
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
  sqlLiteralHelpers?: string[];
  /** Shared policy: report and advise emit advice; ignore suppresses diagnostic advice. */
  unanalyzable?: Unanalyzable;
  /** Legacy alias; fail emits the same advice as advise. Canonical unanalyzable wins. */
  unanalyzableSql?: PostgresGroupingNotAggregationUnanalyzableSql;
}
