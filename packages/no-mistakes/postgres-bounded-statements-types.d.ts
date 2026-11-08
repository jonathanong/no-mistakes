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
  statements?: ("select" | "update" | "delete")[];
  allow?: { object: string; reason: string }[];
  viewBounds?: Record<string, PostgresViewBoundMode>;
  /** Catalog `table.column` names, each with a nonempty reason. */
  columnBounds?: Record<string, PostgresColumnBound>;
  /** Independent pool-body bounds, excluding existence/correlated scalar probes. Defaults to ignore; require-bound advises. */
  materializedBodies?: PostgresMaterializedBodyPolicy;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
