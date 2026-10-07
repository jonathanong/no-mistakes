# PostgreSQL source API

`parsePostgresSql` returns versioned, typed facts from SQL text. It runs on the
native async worker without repository discovery, filesystem reads, invocation
locks, PostgreSQL connections, or a database catalog.

```js
import { parsePostgresSql } from "no-mistakes";

const facts = await parsePostgresSql({
  sql: "CREATE TABLE app.accounts (id uuid PRIMARY KEY);",
  fileName: "migration.sql",
});
for (const statement of facts.statements) {
  if (statement.kind === "createTable") {
    console.log(statement.table, statement.columns, statement.constraints);
  }
}
console.log(facts.diagnostics);
```

Passing an array of `PostgresSqlSource` inputs returns an array of facts in input
order. Independent sources run in parallel. The optional `fileName` is an opaque
label; it is never opened. Each source owns one prepared token inventory and each
statement is parsed once. Projections borrow that statement's AST, which is
discarded after its serializable facts are collected. There is no persistent
parser cache or competing parser engine.

## Contract

`PostgresSqlFacts.schemaVersion` is `1`. All public fact names are exported from
the package declarations, including `PostgresSqlStatement`,
`PostgresSqlStatementKind`, `PostgresSqlColumn`, `PostgresSqlType`,
`PostgresSqlConstraint`, `PostgresSqlAlterOperation`, `PostgresSqlIndex`,
`PostgresSqlView`, `PostgresSqlTrigger`, `PostgresSqlFunction`, and
`PostgresSqlDrop` and `PostgresSqlProceduralBlock`. Statements use a discriminated `kind`; consumers do not need
raw parser nodes or internal imports.

- CREATE TABLE columns retain quoted/qualified type names, array dimensions,
  numeric and character modifiers, defaults, generated expressions, identities,
  and column/table constraints. Expression facts include column and function
  references. Builtin `sql`/`builtin` spellings come from the parsed AST; custom
  types expose their individual name parts.
- ALTER TABLE facts expose added columns and constraints, changed column types
  with USING expressions, defaults, nullability changes, and named constraint
  validation. Other operations
  retain their SQL spelling instead of pretending to implement migration policy.
- CREATE INDEX facts retain relation, method, ordered expressions, operator
  classes, INCLUDE columns, uniqueness, NULLS DISTINCT, options and predicates.
  `structuralIdentity` excludes the index name and source locations, folds
  unquoted expression names, and resolves sort defaults: ascending implies
  NULLS LAST, descending implies NULLS FIRST. Quoting, string contents,
  expressions, predicates and meaningful ordering differences remain distinct.
  Identity is versioned with the facts; it is not a promise of catalog-assisted
  semantic equivalence or arbitrary SQL algebra simplification.
- `CREATE RECURSIVE VIEW` and `CREATE OR REPLACE RECURSIVE VIEW`, including
  explicit column lists, emit `createView` facts with `materialized: false`.
  Declared identifier parts, column names, replacement flags and original
  statement SQL/spans are preserved. The query retains the declared query body;
  the API does not expand the recursive-view shorthand into a recursive CTE.
  `TEMP`/`TEMPORARY` headers preserve `temporary: true`. Recursive views require
  a nonempty column-name list; omission produces a diagnostic. Dependency facts
  model the implicit recursive binding, excluding unqualified self-references
  while retaining qualified physical references. The shared strict and lenient
  parser ASTs carry an equivalent recursive CTE for downstream statement and
  lifecycle analysis; source API query text retains the declared body.
- Views and materialized views expose relation dependencies and typed function
  references with source spans for drop/recreation ordering.
  CTE references follow their PostgreSQL visibility scope and are excluded from
  physical relation dependencies. `dependenciesComplete` is false when the
  parser loses source identity and association is ambiguous; a diagnostic also
  explains that gap. Never treat an incomplete dependency list as proof that no
  dependencies exist.
- Trigger facts include relation, timing/events, transition relations, WHEN
  expressions and called function/arguments. Function facts include signature,
  return type, language, body SQL, behavior and execution attributes. Quoted
  bodies remain bodies; the API does not infer their execution or interpret an
  embedded procedural language.
- DROP facts preserve object names, behavior, function signatures, trigger
  relations and statement order. Schema replay, collision handling and
  application naming rules remain consumer policies.

## Coordinates and partial results

`PostgresSqlPosition.offset` counts UTF-8 bytes from zero. `line` and `column`
count Unicode scalar values from one. Span ends are exclusive. Statement `sql`
is sliced from the original input at those offsets, preserving its source
identity; an expression's `sql` is deterministic AST rendering.

Statement ordinals count nonempty source statements, including failed ones.
Syntax errors return diagnostics with available source spans and preserve valid
neighboring statements. Lexical errors preserve the valid prefix and report the
remaining invalid source. Diagnostics are not thrown as a single file-wide
failure. Invalid API input still rejects the promise.

Dollar-quoted and standard single-quoted `DO ... BEGIN ... END` bodies in the built-in `plpgsql`
language expose a `doBlock` fact. Its `bodySpan` identifies the original body;
nested statements retain their original global source coordinates and lexical
order. `bodyEncoding` identifies `dollarQuoted` or `singleQuoted`. A nested
statement’s `sql` retains the original literal encoding; doubled quotes in a
single-quoted body remain doubled in that source slice. Typed expression facts
contain the decoded SQL semantics, suitable for consumer replay policies. These are **procedural source occurrences**, never proof that a statement
executes or a constraint is installed or validated. Consumers choose migration
policy separately. Each distinct body owns one token inventory and each nested
SQL statement is parsed once with the same SQL parser. Nested programs have a
bounded parser safety limit.

`PostgresSqlProceduralBlock.complete` is false when body facts are incomplete,
including an incomplete nested program. Inspect its `diagnostics` before using
its occurrence list. If a conditional statement's parser span cannot identify
a nonempty source range, the block returns a diagnostic and remains incomplete;
valid top-level statements before and after it are preserved.
Typed `IF`, `ELSIF`, and `ELSE` blocks expose a `conditional`
fact with ordered `PostgresSqlConditionalBranch` entries. A branch retains its
condition expression (null for ELSE), source span, and nested statement facts.
All branches describe possible source occurrences; the API does not evaluate
conditions or claim that their DDL executes. Declarations, loops, exception
handlers, other procedural languages and escape-string DO bodies remain
explicitly unsupported; no DDL is guessed behind unsupported control flow. Ordinary syntax errors preserve
parseable neighboring body statements. Function bodies remain opaque.

A parser compatibility normalization with a source boundary that cannot be
mapped (for example a synthetic COPY-data terminator) produces a diagnostic
rather than fabricated source positions. Unqualified right-hand TABLE arms preserve their original AST identity and
statement delimiters, including quoted names and following statements. Source
boundaries reuse original prepared-token spans; compatibility padding does not
create synthetic source text. Other unsupported TABLE grammar still produces
explicit diagnostics. Consumers needing unsupported grammar
must retain that functionality until its corresponding support lands.

Rust callers use `parse_postgres_source(&PostgresSqlSource)` or
`parse_postgres_sources(&[PostgresSqlSource])`. Node callers use the async public
export rather than shelling out or dispatching on raw AST kinds.

PostgreSQL 18 permits `GENERATED ALWAYS AS (...) VIRTUAL` and makes virtual storage
the default ([generated-column documentation](https://www.postgresql.org/docs/18/ddl-generated-columns.html)).
The selected parser requires a stored-mode token. The source API prepares a
compatibility token at the original expression boundary and restores the
declared or default virtual mode in that same parsed AST before projecting
facts. Explicit `STORED` remains stored, and original statement SQL and source
positions remain unchanged. No alternate dialect or second AST parse is used.

### Parenthesized FETCH counts

The shared PostgreSQL parser accepts expression counts such as
`FETCH FIRST (COALESCE(NULL, 100)) ROWS ONLY`. It preserves the original quantity
AST, Unicode source coordinates, offsets, and `WITH TIES` policy. The prepared
token adapter uses the same parser's LIMIT expression grammar and restores FETCH
on that AST; it does not substitute a literal cap or parse the statement again.

## Expression roots and direct arguments

`PostgresSqlExpression.root` is a discriminated `PostgresSqlExpressionRoot`.
A direct `uuid_generate_v7()` default has `kind: "functionCall"`; a CASE
containing that function has a `case` root (possibly inside `parenthesized`).
An arithmetic expression containing `uuid_extract_timestamp(id)` has a
`binary` root. Flat `columns` and `functions` still list nested occurrences
for dependency analysis and do not establish the root expression.

Parentheses and casts remain explicit wrappers with an `expression` field.
Consumers may unwrap those two kinds to recognize a cast-wrapped root call.
No other root kind promotes a contained function call. `columnReference`
retains each quoted/qualified name component; `literal`, `unary`, `binary`,
`case`, `subquery`, and `other` remain distinct.

A `functionCall` retains its exact name, ordered `PostgresSqlCallArgument`
records, `argumentsComplete`, `syntax`, and call `modifiers`. Argument roots
distinguish `f(id)`, `f(id + 1)`, `f(g(id))`, and `f('id')`; repeated arguments
remain repeated. Named arguments retain the name. Argument `sql` is rendered
SQL, while spans refer to the original UTF-8 source. Wildcards, subquery
argument lists, and expression-named arguments set `argumentsComplete: false`;
consumers requiring a fully supported direct-argument shape must fail closed.
Call modifiers retain DISTINCT, ordering, FILTER, OVER, null treatment, and
aggregate clauses so a plain call policy can require an empty list.

Bare `CURRENT_TIMESTAMP`, `CURRENT_DATE`, and similar SQL value functions
have `syntax: "value"`; parenthesized `CURRENT_TIMESTAMP(3)` has
`syntax: "call"`. These are syntactic facts, not catalog or volatility claims.
Function-name expectations and permitted argument forms remain consumer policy.
Structural `identity` and flat references preserve their existing behavior.

## SELECT scope facts

`parsePostgresSql()` projects query statements as `{ kind: "select", query }`.
`PostgresSqlQuery` contains deterministic, query-local IDs for scopes, relations,
joins, CTE definitions, column references, equality predicates, and EXISTS
occurrences. It uses the already parsed AST and requires no project root or
schema catalog. Existing statement ordinals, SQL, structural identities and
source spans retain their meaning.

Scopes record their parent and hosting clause: projection, FROM, WHERE, JOIN ON,
HAVING, GROUP BY, ORDER BY, LIMIT or OFFSET. Set operations own separate ordered
branch scopes and expose their operator and quantifier. Names preserve quoted
identifier identity and schema qualification; relation aliases hide their
underlying table names. Joined-group aliases hide the individual participants.
Derived tables expose their child scope, lateral status, and declared column
aliases. This is syntactic ownership, not catalog-backed column lineage.

Qualified columns resolve against visible relation aliases in the current
scope, then permitted outer scopes. Non-lateral derived tables and CTE bodies
cannot see the containing query's FROM aliases. Unqualified, unknown and
ambiguous columns keep explicit resolution statuses and null relation IDs.
No relation is guessed from an unqualified column name.

CTEs record lexical ownership, declared columns, recursive WITH status,
references, use reachable from the main query, and reference cycles. Unused
CTE definitions still have facts. Inner definitions shadow outer names;
non-recursive definitions see preceding CTEs, while recursive WITH definitions
can reference the complete local group. Cycle reporting describes syntax and
does not validate PostgreSQL's recursive-query restrictions.

Joins record ordered left/right relation IDs, inner/outer/cross/semi/anti kind,
and ON, USING, NATURAL or absent constraints. Equality facts retain their
clause, join ID, direct column operands when available, and predicate context.
Only positive conjuncts in WHERE and inner/semi JOIN ON have `mandatory: true`.
OR, NOT, CASE, boolean tests, function arguments and other expression wrappers
retain conservative context flags. Outer JOIN ON and HAVING equalities are
never promoted to mandatory predicates. Consumers must interpret these facts
for their own policies; the parser does not prove authorization or tenant
isolation.

EXISTS facts expose negation, context, child scope and resolved references to
outer relations. `correlated` describes those known qualified references;
unqualified references cannot establish correlation without a catalog.

Unsupported relation, expression or query forms appear in `unsupported` with
a reason, hosting scope/clause and available source span, and set
`complete: false`. Table functions, named windows, window
frames and function argument clauses currently take this path. Parse failures
remain source diagnostics. Spans use UTF-8 byte offsets and Unicode scalar
line/column positions, including quoted identifiers and nested queries.

## INSERT and ON CONFLICT facts

`kind: "insert"` exposes `insert.table`, the optional target `alias`, explicit
`columns`, and `columnsOmitted`. Names retain quoted identity. The `source` union
identifies `values` (typed expression rows), `select` (the same scope facts used
for SELECT), `defaultValues`, or `unsupported`. Statement spans and nested
expression spans refer to the original input, including comments and literals.

`onConflict` is null when absent. Its `target` distinguishes `omitted`, `columns`,
and `constraint`. The conflict-target `predicate` is separate from the optional
`doUpdate` action predicate. Actions are `doNothing` or `doUpdate`, whose
`assignments` expose target columns, typed expressions, spans, and `provenance`:
`targetColumn`, `excludedColumn`, `literal`, `placeholder`, or `unresolved`.
Parentheses and casts preserve the underlying reference provenance; function
calls, arithmetic, subqueries, tuple assignments, and unknown qualifiers remain
unresolved. An alias hides the original target name. This is syntax provenance,
not database type checking or a replay-safety decision.

```js
const facts = await parsePostgresSql({
  sql: "INSERT INTO accounts (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id",
});
const { insert } = facts.statements[0];
console.log(insert.onConflict.action.assignments[0].provenance); // excludedColumn
```

Consumers must check `insert.complete`, `insert.diagnostics`, assignment
`complete`, and SELECT source completeness before relying on facts. Unsupported
INSERT extensions (including RETURNING, whose output facts are outside this
contract) and unresolved assignment provenance set completeness false and
produce diagnostics. Syntax the prepared parser cannot represent produces the
existing source diagnostic and preserves neighboring valid statements. No raw
parser AST, embedded-language extraction, or consumer policy is exported. All
INSERT contracts are named exported TypeScript types available from both Node
facades. The asynchronous API reuses its prepared token stream and parses each
expression once, including partial-index conflict predicates.

An outer `WITH` on `INSERT ... SELECT` is retained in the source query's CTE
facts. Outer CTEs with `VALUES` or `DEFAULT VALUES`, and overlapping outer and
source-level `WITH` scopes, currently produce incomplete INSERT facts. Signed
numeric constants have literal assignment provenance; unary expressions over
columns remain unresolved. Procedural blocks inherit incomplete INSERT facts,
including INSERT occurrences inside conditional branches.

`VALUES` source facts expose rows. Query-level CTEs, ordering, pagination,
locking, and other query modifiers that are absent from that row contract mark
the INSERT incomplete rather than silently discarding their meaning.
