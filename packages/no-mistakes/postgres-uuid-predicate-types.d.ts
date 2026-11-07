import type { Effort, Unanalyzable } from "./traversal-types";

/** Legacy SQL policy spelling; use Unanalyzable for the shared policy. */
export type PostgresUuidPredicateUnanalyzableSql = "fail" | "ignore" | "advise";

/** Configuration for `postgres-no-transformed-uuid-predicates`. */
export interface PostgresNoTransformedUuidPredicatesOptions {
  /** Required unless inherited from the selected project PostgreSQL scope. */
  schemaCatalogPath?: string;
  include?: string[];
  exclude?: string[];
  sqlInclude?: string[];
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
  unanalyzableSql?: PostgresUuidPredicateUnanalyzableSql;
}
