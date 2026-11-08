/** The predicate proved by a PostgreSQL catalog CHECK constraint. */
export type PostgresColumnCheckPredicate = "non-negative";
export type PostgresColumnCheckIntegerType =
  | "smallint"
  | "integer"
  | "bigint"
  | "int"
  | "int2"
  | "int4"
  | "int8";
export interface PostgresColumnCheckRequirement {
  /** Case-sensitive glob matched against column names. */
  pattern: string;
  types: PostgresColumnCheckIntegerType[];
  require: PostgresColumnCheckPredicate;
}
export interface PostgresColumnCheckAllowEntry {
  /** Exact table.column catalog identity. */
  column: string;
  /** Nonempty explanation for the intentional exception. */
  reason: string;
}
/** Options for the postgres-column-check-constraints rule; use application severity for adoption. */
export interface PostgresColumnCheckConstraintsOptions {
  /** Omit to leave the rule inactive. */
  schemaCatalogPath?: string;
  columns?: PostgresColumnCheckRequirement[];
  allow?: PostgresColumnCheckAllowEntry[];
}
