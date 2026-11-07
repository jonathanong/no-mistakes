import type { Effort, Unanalyzable } from "./traversal-types";

/** A throwing helper that validates one finite SQL identifier domain. */
export interface PostgresSqlIdentifierValidator {
  module?: string;
  name: string;
  argument: number;
}

/** Configuration for the `postgres-no-unbounded-sql-identifier` check rule. */
export interface PostgresNoUnboundedSqlIdentifierOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  finiteDomainAssertions?: { name: string; argument: number }[];
  identifierValidators?: PostgresSqlIdentifierValidator[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
