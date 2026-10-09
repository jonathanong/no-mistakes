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
`decodedSql` owns nested command statement, expression, and diagnostic coordinates.
Children preserve order and typed facts (including INSERT), without exposing an
AST or implying execution. Inspect `complete` and diagnostics: malformed nested
SQL remains diagnostic. Wholly literal `||` concatenations use the same nested
pipeline and expose `bodyEncoding: "concatenated"`; `literalSpan` covers the full
command expression. `using` retains ordered parameter expressions and their
original source spans, without resolving runtime values; `$1` and other command
placeholders remain typed parameters. Dynamic operands, `format` calls, and
unsupported EXECUTE modifiers remain incomplete `other` occurrences. This adds no SQL execution or
replay policy. `parsePostgresSql` retains its asynchronous single/batch API.

INSERT assignment `complete` describes represented syntax independently of
`provenance`. Function expressions such as `COALESCE(t.v, EXCLUDED.v)` expose
`derived` provenance and complete syntax. Unresolved column lineage does not
itself imply unsupported syntax; inspect diagnostics for incomplete projections.
Unary roots expose a typed `expression` operand, including signed literals;
unsupported operand projections remain incomplete. These facts do not establish
SQL execution or replay safety.

Expression roots distinguish `nullTest` (`negated` for IS NOT NULL),
`distinctness` (`negated` for IS NOT DISTINCT FROM), `parameter` (`placeholder`,
including `$1`), and `typedLiteral` (`dataType`, decoded string `value`, rendered
`sql`). Recursive children retain `nullOperand` and ordered `distinctLeft` /
`distinctRight` operands under operators, calls, CASE, and wrappers. Temporal
literals retain their type, precision, timezone qualifier, and value: `now`,
`today`, and `epoch` are syntax facts, without a volatility or replay policy.
`childrenComplete` describes represented syntax independently of source spans.
AND/OR trees, null tests, and qualified casts are complete when every operand is
represented. A null span still means that its exact source boundary is unknown;
it does not erase complete structure. Unsupported expressions, omitted children,
and depth limits remain explicitly incomplete and callers must fail closed.
Ordinary `literal` roots also expose `value: PostgresSqlLiteralValue`,
distinguishing SQL null, string, number, and boolean values. Number values retain their decimal spelling as
strings; quoted and escaped string values use the parser-decoded contents.
Unclassified values expose `other` with SQL rather than guessed semantics.

Expression `children` preserve immediate operand order and role, then recurse;
check `childrenComplete` before relying on the whole expression tree. Explicit
INSERT columns expose positional `columnSources` across VALUES rows and query
branches. Unsupported source shapes carry a reason and do not produce partial
maps. Spans can be null where the prepared tokens cannot prove complete wrapper
boundaries.

`CREATE INDEX ... ON ONLY relation` is represented by `index.only: true`;
quoted or expression-level `ONLY` identifiers remain part of their relation,
key, or predicate facts.

Expression conflict arbiters expose an ordered `target.expressions` list.
Subscripted assignments expose `target.base`, ordered `target.subscripts`, and
source spans while preserving base names in `columns` and RHS provenance. Use
these as syntax facts; do not infer catalog index matches or replay safety.

Arbiter `operatorClasses` aligns with `target.expressions` when a class is
specified. Mixed array/composite targets retain ordered `target.indirection`
steps, while nested call and argument spans include their closing delimiters.

The extended conflict grammar applies to prepared outer INSERT clauses, including
EXPLAIN/PREPARE wrappers. Expression arbiters and subscripted assignments inside
data-modifying CTE INSERTs currently diagnose through the upstream grammar;
neighboring statements remain available. Exact closing-delimiter spans apply
to the prepared conflict arbiters, assignment targets/RHS, and target/action
predicates; AST-only nested conditional/CTE projections retain existing span limits.
Operator classes expose optional ordered typed `parameters` (name and value),
including their full class source span. Leading composite fields enter the
ordered assignment indirection after its base column.

Use `PostgresSqlConstraint.span` to locate CREATE inline/table constraints and
ALTER ADD COLUMN inline constraints or ADD CONSTRAINT in their owning source.
Named ranges include `CONSTRAINT`;
ALTER ranges exclude `ADD` and include `NOT VALID`. Coordinates refer to original
SQL, or `decodedSql` for literal EXECUTE children. A null span means the token
boundary is unavailable; do not guess offsets from formatted `constraint.sql`.

Use trigger `eventFacts` for event kinds and ordered `updateColumns`; `updateOf`
distinguishes restricted from unrestricted UPDATE. Compare cast
`dataTypeFacts.name.parts` or `builtin` rather than parsing `dataType` display
text. Literal EXECUTE `using` expressions are source syntax, never resolved
runtime parameters. Require `childrenComplete` for represented structure and
check nullable spans separately before relying on exact source boundaries.
