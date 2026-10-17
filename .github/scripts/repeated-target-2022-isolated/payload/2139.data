/** Policy for executor SQL whose leading text cannot be proven. Defaults to `report`. */
export type PostgresUnanalyzableSql = "report" | "ignore";

/** Configuration options for the `postgres-require-query-annotation` check rule. */
export interface PostgresRequireQueryAnnotationOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  unanalyzableSql?: PostgresUnanalyzableSql;
}
