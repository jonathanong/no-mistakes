import type { Effort, Unanalyzable } from "./traversal-types";

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
  unanalyzableSql?: "fail" | "ignore" | "advise";
}
