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

Literal PL/pgSQL `EXECUTE` source occurrences in supported `DO` bodies expose
`kind: "literalExecute"` and a `PostgresSqlLiteralExecute` payload. Dollar-quoted,
standard single-quoted (doubled quotes), and PostgreSQL `E` escape strings are
decoded by the prepared tokenizer and parsed through the same SQL fact pipeline.
The enclosing statement and `literalSpan` retain original source coordinates;
`decodedSql` owns all child statement, expression, and diagnostic coordinates.
Children preserve order and typed facts (including INSERT), without exposing an
AST or implying execution. Inspect `complete` and diagnostics: malformed nested
SQL remains diagnostic; dynamic variables, concatenation, `format`, and EXECUTE
modifiers remain unsupported `other` occurrences. This adds no SQL execution or
replay policy. `parsePostgresSql` retains its asynchronous single/batch API.

INSERT assignment `complete` describes represented syntax independently of
`provenance`. Function expressions such as `COALESCE(t.v, EXCLUDED.v)` expose
`derived` provenance and complete syntax. Unresolved column lineage does not
itself imply unsupported syntax; inspect diagnostics for incomplete projections.
Unary roots expose a typed `expression` operand, including signed literals;
unsupported operand projections remain incomplete. These facts do not establish
SQL execution or replay safety.
