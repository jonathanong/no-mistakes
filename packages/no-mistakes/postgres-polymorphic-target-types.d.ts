export interface PostgresPolymorphicTargetAllowEntry {
  /** Exact table.column catalog identity for either column in the polymorphic pair. */
  column: string;
  /** Nonempty explanation for the intentional exception. */
  reason: string;
}
/** Options for the postgres-polymorphic-target rule. */
export interface PostgresPolymorphicTargetOptions {
  /** Repository-relative path to the complete PostgreSQL schema catalog. */
  schemaCatalogPath: string;
  /** Column name suffix for the discriminator column (default: "_type"). */
  typeSuffix?: string;
  /** Column name suffix for the target identifier column (default: "_id"). */
  idSuffix?: string;
  allow?: PostgresPolymorphicTargetAllowEntry[];
}
