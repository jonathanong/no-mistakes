export interface PostgresCatalogOptions {
  /** Environment variable containing a PostgreSQL connection URL; never pass secrets as options. */
  connectionEnv: string;
  /** Exact PostgreSQL schema name. */
  schema: string;
}
export interface PostgresOrderingCatalog {
  formatVersion: 2;
  coverage: "ordering";
  schema: string;
  tables: Record<string, PostgresCatalogTable>;
}
export interface PostgresCatalogColumn {
  dataType: string;
  nullable: boolean;
  ordinalPosition: number;
  defaultExpression: string | null;
  generated: "stored" | "virtual" | null;
  generatedExpression: string | null;
  identity: "a" | "d" | null;
}
export interface PostgresCatalogIndexKey {
  column: string | null;
  opclass: string;
  collation: string | null;
  orderingSupported: boolean;
  expression: string;
  descending: boolean;
  nullsFirst: boolean;
}
export interface PostgresCatalogIndex {
  accessMethod: string;
  unique: boolean;
  primary: boolean;
  constraintBacked: boolean;
  immediate: boolean;
  live: boolean;
  valid: boolean;
  ready: boolean;
  predicate: string | null;
  definition: string;
  keys: PostgresCatalogIndexKey[];
}
export interface PostgresCatalogTable {
  relationKind: "table" | "partitioned table";
  columns: Record<string, PostgresCatalogColumn>;
  primaryKey: { columns: string[] } | null;
  uniqueConstraints: Record<string, { columns: string[] }>;
  indexes: Record<string, PostgresCatalogIndex>;
}
