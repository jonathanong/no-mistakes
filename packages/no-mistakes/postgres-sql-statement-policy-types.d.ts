import type { Effort, Unanalyzable } from "./traversal-types";

export type PostgresSqlStatementPolicyClause = "create-extension-version" | "drop-cascade";

/** Configuration for the `postgres-sql-statement-policy` rule. */
export interface PostgresSqlStatementPolicyOptions {
  include?: string[];
  exclude?: string[];
  sqlInclude?: string[];
  /** Omit for the default statement kinds, or pass [] to disable statement-kind checks. */
  bannedStatements?: string[];
  bannedSettings?: string[];
  /** Exact clause names to ban. Defaults to []. */
  bannedClauses?: PostgresSqlStatementPolicyClause[];
  importSpecifier?: string;
  executorNames?: string[];
  extraExecutorNames?: string[];
  excludeExecutorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: "fail" | "ignore" | "advise";
}
