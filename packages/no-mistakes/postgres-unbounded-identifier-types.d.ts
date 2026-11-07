import type { Effort, Unanalyzable } from "./traversal-types";

/** A trusted throwing validator for one SQL identifier argument and its returned value. */
export interface PostgresSqlIdentifierValidator {
  module?: string;
  name: string;
  argument: number;
}

/** A trusted helper whose scalar return is a bounded SQL identifier. */
export interface PostgresSqlIdentifierSource {
  module?: string;
  name: string;
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
  identifierSources?: PostgresSqlIdentifierSource[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: Unanalyzable;
}
