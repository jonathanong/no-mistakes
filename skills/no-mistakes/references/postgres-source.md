# Parse PostgreSQL source without a repository

Use the async Node `parsePostgresSql({ sql, fileName })` export for migration
text and schema-authoring facts. Pass an array for a source batch. It needs no
root, database, catalog, invocation lock, or raw AST traversal.

Inspect `diagnostics` and each view's `dependenciesComplete` before treating
the facts as a complete dependency inventory. Procedural bodies and unsupported
grammar are explicit limitations; do not infer migration execution from raw
body text. UTF-8 offsets are zero-based; Unicode line/columns are one-based,
with exclusive span ends. See [the source API](https://github.com/jonathanong/no-mistakes/blob/main/docs/postgres-source-api.md)
for exported types, structural index identity, and grammar boundaries.
