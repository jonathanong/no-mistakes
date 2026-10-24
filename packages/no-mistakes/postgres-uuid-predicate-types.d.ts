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
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: PostgresUuidPredicateUnanalyzableSql;
}
