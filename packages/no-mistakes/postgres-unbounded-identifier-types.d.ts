import type { Effort, Unanalyzable } from "./traversal-types";

/** Configuration for the `postgres-no-unbounded-sql-identifier` check rule. */
export interface PostgresNoUnboundedSqlIdentifierOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  /** Replaces default executor names; mutually exclusive with extraExecutorNames. */
  executorNames?: string[];
  /** Extends query/read/write; mutually exclusive with executorNames. */
  extraExecutorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  finiteDomainAssertions?: { name: string; argument: number }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
