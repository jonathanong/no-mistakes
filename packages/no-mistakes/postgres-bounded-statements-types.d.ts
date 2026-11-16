import type { PostgresRawSqlTag } from "./postgres-raw-sql-tag-types";
import type { PostgresSqlIdentifierAssertion } from "./postgres-sql-identifier-assertion-types";
import type { Effort, Unanalyzable } from "./traversal-types";

export type PostgresMaterializedBodyPolicy = "ignore" | "advise" | "report" | "require-bound";

export type PostgresViewBoundMode = "bounded" | "unbounded";

/** An application assertion that each selected key has bounded fan-out. */
export interface PostgresColumnBound {
  reason: string;
}

/** Configuration for the `postgres-bounded-statements` check rule. */
export interface PostgresBoundedStatementsOptions {
  schemaCatalogPath: string;
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
  sqlIdentifierAssertions?: PostgresSqlIdentifierAssertion[];
  rawSqlTags?: PostgresRawSqlTag[];
  statements?: ("select" | "update" | "delete")[];
  allow?: { object: string; reason: string }[];
  viewBounds?: Record<string, PostgresViewBoundMode>;
  /** Catalog `table.column` names, each with a nonempty reason. */
  columnBounds?: Record<string, PostgresColumnBound>;
  /** Independent pool-body bounds, excluding existence/scalar subquery probes. Defaults to ignore; require-bound advises. */
  materializedBodies?: PostgresMaterializedBodyPolicy;
  /**
   * Accept a unique-key lookup when every free column has a proven finite domain
   * and the product of those domains is at most this many rows. Unset reports
   * those lookups. An integer type's width is not a domain.
   */
  maxDomainRows?: number;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
