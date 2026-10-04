# Parse PostgreSQL source without a repository

Use the async Node `parsePostgresSql({ sql, fileName })` export for migration
text and schema-authoring facts. Pass an array for a source batch. It needs no
root, database, catalog, invocation lock, or raw AST traversal.

Inspect `diagnostics` and each view's `dependenciesComplete` before treating
the facts as a complete dependency inventory. Dollar-quoted and standard single-quoted DO blocks
expose nested source occurrences and original body spans; inspect each block's
`complete`, `bodyEncoding`, and `diagnostics`, and never treat its statements as guaranteed
execution. IF/ELSIF/ELSE branches expose typed conditions and nested statements;
inspect every branch without assuming its condition is true. PostgreSQL 18 virtual generated columns retain their declared or
default storage. Unsupported procedural control flow remains explicit. UTF-8 offsets are zero-based; Unicode line/columns are one-based,
with exclusive span ends. See [the source API](https://github.com/jonathanong/no-mistakes/blob/main/docs/postgres-source-api.md)
for exported types, structural index identity, and grammar boundaries.
