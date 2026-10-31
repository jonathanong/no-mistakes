import type { Effort, Unanalyzable } from "./traversal-types";

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
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  statements?: ("select" | "update" | "delete")[];
  allow?: { object: string; reason: string }[];
  viewBounds?: Record<string, PostgresViewBoundMode>;
  /** Catalog `table.column` names, each with a nonempty reason. */
  columnBounds?: Record<string, PostgresColumnBound>;
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
