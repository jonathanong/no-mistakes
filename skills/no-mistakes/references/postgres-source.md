# Parse PostgreSQL source without a repository

For executed TypeScript SQL, use `check` with configured
`postgres-require-query-annotation` instead. At default `effort: max`, it traces straight-line local/imported
helpers and callback forwarding, and reports unknown leading SQL by default.
Inspect unanalyzable findings before using `unanalyzable: ignore` (or its
legacy alias `unanalyzableSql: ignore`); a static
leading `/* name */` can establish the annotation even when later appends are opaque.

At `effort: low`, annotation checking skips the annotation evaluator. It proves
literal heads at the executor call or in one local const, trusted
`sql-template-strings` append prefixes, finite all-named conditional arms,
and fixed-return helpers (including total finite returns and one safe returned tagged local) in the same
file or through one direct source import. See the
[bounded syntax shapes](https://github.com/jonathanong/no-mistakes/blob/main/docs/postgres-query-annotation/low-effort.md).
Low also accepts a literal string `const` name inside the first comment and a
closed `as const` literal table member passed directly to an executor. Table
writes, aliases, exports, and dynamic reads keep the unanalyzable policy.
An exact top-level namespace destructure can follow a unique explicit source
barrel export to that helper; dynamic keys and ambiguous exports abstain.
A helper may return one trusted local builder even when captured closures only
append suffixes; every reference must be a discarded append, proven append-only
effect, or direct return from that helper.
Low also proves a first ordinary comment name from up to 16 direct string
tuples in a `for-of` loop, a plain ternary of two string literals, a
synchronous `for (const name of ['one', 'two'])`, or a closed set of direct
literal calls through one local select closure into a same-file helper. An
unresolved import anywhere in the file leaves the ternary and string-list
forms under `unanalyzable`. Unknown names and tagged
bind interpolation remain unanalyzable.
Other helper, alias, and mutation shapes use
`unanalyzable: report | advise | ignore`. `trustedSqlTags` is deprecated and
ignored for this rule at low effort.
Captured local trusted builders retain their named prefix when every use of the
same binding is a discarded append, configured executor read, or proven local
append-only effect. Closure order does not establish complete SQL.
Discarded helper calls may compose through exact same-file append-only helpers
for at most four local edges. A longer chain leaves that helper effect unresolved,
so the builder head is unresolved and reported under `unanalyzable`.
A returned parameter append chain qualifies only when its caller discards the
result. Used aliases and nested captures stay unanalyzable.

Use the async Node `parsePostgresSql({ sql, fileName })` export for migration
text and schema-authoring facts. Pass an array for a source batch. It needs no
root, database, catalog, invocation lock, or raw AST traversal.

Inspect `diagnostics` and each view's `dependenciesComplete` before treating
the facts as a complete dependency inventory. Dollar-quoted and standard single-quoted DO blocks
expose nested source occurrences and original body spans; inspect each block's
`complete`, `bodyEncoding`, `occurrences`, and `diagnostics`, and never treat its statements as guaranteed
execution. `occurrences` distinguish `utility` (`CREATE TYPE`), `controlFlow`
(`IF`/`RAISE` with no DML), nested `dml` (`INSERT`/`UPDATE`/`DELETE`/`MERGE`
inside `IF` or `LOOP`), `dynamicExecute`, and `unknown`. A `dml` entry is
visible source text, not an executed statement. Dynamic `EXECUTE` fails closed.
IF/ELSIF/ELSE branches expose typed conditions and nested statements;
inspect every branch without assuming its condition is true. PostgreSQL 18 virtual generated columns retain their declared or
default storage. Unrecognized procedural forms remain explicit. UTF-8 offsets are zero-based; Unicode line/columns are one-based,
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
unsupported EXECUTE modifiers remain incomplete `other` occurrences. An unquoted `INTO` target ends the command expression the same way `USING` does: the occurrence is classified from the literal, and the block stays incomplete. This adds no SQL execution or
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
EXPLAIN/PREPARE wrappers. A column-target `WHERE` inside a data-modifying CTE
INSERT is projected, including `DO NOTHING`, `DO UPDATE`, and `RETURNING`, and
the same predicate is kept when that CTE is the child of an EXPLAIN or PREPARE
wrapper.
Expression arbiters and subscripted assignments inside data-modifying CTE
INSERTs still diagnose through the upstream grammar; neighboring statements
remain available. Exact closing-delimiter spans apply
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

Scalar array `ANY`/`ALL` comparisons appear in `query.quantifiedComparisons`,
in traversal order, with `scopeId`, `clause`, `joinId`, operator, `quantifier`
(`any` or `all`), predicate context, nullable source span, and typed `left`/`right`
expressions. `complete` requires a supported scalar comparison operator
(`=`, `<>`, `<`, `<=`, `>`, `>=`) and completely represented operands. Columns,
literals (including signed numbers), bind parameters, parentheses, casts, and array literals are supported;
array expressions expose ordered `arrayElement` children. Row operands, subqueries,
arithmetic or unknown function operands, `SOME`, and unsupported operators remain incomplete
with query diagnostics. Quantified equality does not add an ordinary equality
fact or evaluate array contents. An otherwise supported `INSERT ... SELECT`
with `x = ANY(t.aliases)` can therefore be complete.

Grouped residual advice, including a `target` that may list several checks, is described in
[SQL verdicts](https://github.com/jonathanong/no-mistakes/blob/main/docs/postgres-sql-verdicts.md).

For `postgres-no-unbounded-sql-identifier`, low effort accepts direct literal SQL/templates or one lexical const and skips call-graph/provenance inference. Keep relations literal; bind external values. Helper/alias/factory/type shapes are unanalyzable and follow `report`, `advise`, or `ignore`. Direct global `process.env` relation inputs remain violations.

At max effort, configure `identifierValidators: [{module: "@example/sql-identifiers", name: "assertIdentifier", argument: 0}]` for a trusted throwing helper enforcing an allow-list or identifier grammar plus length limit. A completed call proves the argument at later uses and its returned value, including `const table = assertIdentifier(input)`. The body is not inspected. Module identity matches either the call-site import or canonical resolved origin through named/`export *` barrels and npm re-exports. Validate each property before SQL interpolation; a schema assertion does not validate the table. Existing `finiteDomainAssertions` remains supported. Closed `includes`/`has` guards may throw or return, including guards over imported literal `as const` arrays with plain string parameters; a catalog-derived parameter type alone is not runtime validation. Static metadata-array `.find` lookups need a dominating presence guard.

Use `identifierSources: [{module: "@example/relations", name: "getRelationTableNameOrThrow"}]` for an explicit trusted scalar identifier producer when automatic proof does not cover a catalog lookup wrapper. The body is not inspected; this option proves the return, not input arguments or arbitrary object properties. Both helper lists default to empty. Omit `module` only to match an exact local callee name.

Workspace package and `export *` re-exports retain source identity, including typed catalogs filled by module-load loops and literal joins. On a complete max-effort scan, each source entry must supply a scalar identifier proof in selected SQL. A matching helper call with no SQL use, or a property read from its return, does not count. Correct or remove unused selectors reported as `unused-allow-entry` at the config item; `reportUnusedAllowEntries` controls report, advise, or ignore.

Known literal properties selected from a closed record by a cross-module helper and module constants built with closed catalog filter/map/join chains are bounded. Keep catalog fields literal and avoid mutations, escapes, or accessors. An exported UUID helper returning `` `${prefix}_${randomUUID().replaceAll("-", "")}` `` also qualifies when its sole prefix argument is literal at the caller; `await` preserves the name through an async preparation helper. Open prefixes, fake UUID producers, mixed returns, reassignment, recursive returns, and async results used without `await` retain findings.

The default and `SQL` imports from `sql-template-strings` need no tag configuration. At max effort, their `.text` proof includes immutable module-const fragments, `.append()` chains, statement appends, and resolved synchronous helper returns with matching `.values`. This includes imported statements and factories used inside callbacks whose imported executor type is configured with `executorTypeNames`. Avoid reassignment, member writes, and unsafe aliases or escapes in both producer and consumer. Namespace escapes invalidate only statements exposed through actual exports. Copy imported fragments into a fresh `sql` builder with `.append(fragment)`. Use `trustedSqlTags` for custom tag imports. Open raw append values, opaque builders, mutation, and escape do not establish this proof.

A private ESM helper may return a guarded metadata row when every call result is an immutable `const` or a direct known-field projection. Keep all result uses on known literal fields. Any result mutation, alias, method call, helper export, or function-value escape invalidates every helper sharing that metadata source.

`postgres-no-mutable-sql-registry` also supports `effort: low`. Use it when SQL
relations are literal: the syntax tier skips cross-file mutation and call-graph
analysis. Imported registry relations, helpers, and other indirect SQL produce
an unanalyzable diagnostic under `report` (or an advisory under `advise`).
`as const` does not freeze runtime registry members. Choose `max` when the
cross-file registry path needs inspection; see the rule's Effort section.

For annotation checks, configure `sqlLiteralHelpers: [pg-copy-streams#from]` to unwrap a direct imported helper with one string or interpolation-free template literal at low and max effort. Put the SQL name comment inside the literal. Other argument shapes remain unanalyzable; unannotated static literals remain definite findings under every policy. This option is separate from deprecated `trustedSqlTags`.

Low-effort annotation checks can read `.text`/`.values` from a safe local `sql-template-strings` builder and follow one direct imported immutable string constant with a fixed literal head; re-exports and mutable values remain unresolved.
They can also follow one fresh trusted builder returned by a source-visible helper into a local `const` used only by a configured executor and discarded static appends, or through a direct `helper().append(...)` executor argument. Plain records and strings do not acquire builder trust.
An exact imported source helper may forward a fixed named head to a direct inline configured executor callback; unknown callback uses remain under the configured uncertainty policy.

Executor selection uses `executorNames` to replace the default
`[query, read, write]` list. Use `extraExecutorNames: [readPool, writePool]` to
extend it; setting both options is a configuration error. The same options
apply to rule options and `projects.<name>.postgres.executors`. Inspect `check`
configuration warnings for omitted default executors imported from the selected
module and called in scoped files. Fix the list or use `extraExecutorNames`
before interpreting a reduced finding count as resolved work. To omit names
intentionally, use `excludeExecutorNames: [query]`; it removes names after
replacement or defaults plus extras and suppresses omitted-default warnings.
Remove exclusions flagged as unused: they either are not default names or are
never imported from the selected module and called in scoped files. See
[executor selection](https://github.com/jonathanong/no-mistakes/blob/main/docs/configuration/rules.md#postgresql-executor-selection).

Use opt-in `postgres-lock-written-rows-only` to audit configured transaction
executors. Start with its default advisory checks, then set individual check
ids to `fail` after reviewing row, table, constant advisory, and autocommit
locks. The executor replacement, additive, and exclusion options above also
apply to this rule. Configure transaction factories through `executorFactoryNames`
for `const`, `using`, and `await using` bindings. Configure
`executorOptionProperties` when a wrapper routes its transaction through an
inline options object. Directly await static lock helpers and pass matching
executor parameters; imported helpers and re-exports reuse prepared summaries.
Hashed global advisory keys are checked, while atomic lock/write CTEs on the
same relation are accepted. Keep `allow` reasons and `writeSets` assertions
narrow. Unknown SQL, helper execution, or transaction membership follows the
unanalyzable policy and never proves a safe lock. See
[the rule](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/postgres-lock-written-rows-only.md)
for limits and rewrites.

For catalog review, opt into `postgres-materialized-view-shape` with
`schemaCatalogPath`. The shared application `severity` defaults to `error`;
set `severity: advisory` for non-failing findings. It checks for missing
concurrent-refresh unique indexes, additional read indexes
(`minAdditionalIndexes`, default `1`), and unbounded
base-table reads in materialized-view definitions. Configure `windowFunctions`
for key-window predicates or use a plain indexed range; reserve
`allowBaseTables: [{table, reason}]` for bounded-size tables. Run
`no-mistakes check --root <repo> --format json` and inspect `rules` or
`advisories` according to the configured severity. See
the [rule contract](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/postgres-materialized-view-shape.md).
