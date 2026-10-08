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
  PostgreSQL's `ON ONLY relation` form is retained as `index.only: true`.
  `structuralIdentity` excludes the index name and source locations, folds
  unquoted expression names, and resolves sort defaults: ascending implies
  NULLS LAST, descending implies NULLS FIRST. Quoting, string contents,
  expressions, predicates, the `ONLY` relation modifier and meaningful ordering
  differences remain distinct.
  The `only` discriminator is added to structural identity only when true, so
  ordinary index identities keep their existing serialized form.
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
parseable neighboring body statements. String function bodies remain opaque; SQL-language BEGIN ATOMIC declaration wrappers expose child source occurrences without implying execution.

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
retains each quoted/qualified name component; unary roots expose their typed
`expression` operand; `literal`, `unary`, `binary`,
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
`constraint`, and `expressions`. Expression arbiters retain their ordered typed
expressions, function references, identifiers, and source spans. Optional
`operatorClasses` aligns with that list, retaining each named operator class
and its source span (null for entries without a class). Plain column
arbiters retain the existing `columns` shape. The conflict-target `predicate` is
separate from the optional
`doUpdate` action predicate. Actions are `doNothing` or `doUpdate`, whose
`assignments` expose target columns, typed expressions, spans, and `provenance`:
`targetColumn`, `excludedColumn`, `literal`, `placeholder`, `derived`, or
`unresolved`. Parentheses and casts preserve the underlying provenance. Function
calls have `derived` provenance: their value is not one atomic source, even when
all arguments are target/excluded references. Arithmetic, subqueries, tuple
assignments, and unknown qualifiers remain unresolved. An alias hides the original
target name. This is syntax provenance, not database type checking or a
replay-safety decision.

Each expression also exposes ordered immediate `children` with typed `role`,
optional argument/CASE index, a shallow typed `root`, and recursive descendants.
`childrenComplete` reports whether the full expression structure is represented;
it is independent of INSERT syntax diagnostics and lineage, and is also false
when a child's full source boundary cannot be proven. Nullable spans retain the
typed facts that remain available without guessing a boundary. Some AST-only
projections lack a prepared delimiter catalog, so their recursive spans can be
null and `childrenComplete` false even when typed child facts are available.
For INSERTs with an explicit target column list, `columnSources` maps each target
position to every VALUES row or SELECT/set-operation branch in source order.
Set branches use `branchPath` (`0` for left, `1` for right); VALUES entries also
include `rowIndex`. Omitted target columns, DEFAULT VALUES, wildcard/BY NAME
projections, duplicate targets, unsupported source forms, and arity mismatches
return a typed `unsupported` result instead of a partial map. Mapping
`complete` describes recursive expression representation, not SQL validity or
replay safety. INSERTs delegated to a modifying CTE report
`reason: "cteSourceDelegated"` because the query projection owns that source.

Subscripted and field-only composite assignment targets additionally expose `target.base`, ordered
`target.subscripts`, and `target.span`; nested index expressions retain exact
source spans. Mixed array/composite targets additionally expose ordered
`target.indirection` steps (`subscript` and `field`), including quoted field
identity and step spans. Prepared conflict expressions retain complete closing-delimiter spans for
nested function references and call arguments even inside compound expressions.
Base-column names remain in `columns`. These are syntax facts:
no catalog index selection, assignment target resolution, or replay policy is
inferred. Array slices currently diagnose rather than yielding a partial target.

The extended conflict grammar applies to prepared outer INSERT clauses, including
EXPLAIN/PREPARE wrappers. Expression arbiters and subscripted assignments inside
data-modifying CTE INSERTs currently diagnose through the upstream grammar;
neighboring statements remain available. Exact closing-delimiter spans apply
to the prepared conflict arbiters, assignment targets/RHS, and target/action
predicates; AST-only nested conditional/CTE projections retain existing span limits.
Operator classes expose optional ordered typed `parameters` (name and value),
including their full class source span. Leading composite fields enter the
ordered assignment indirection after its base column.

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
contract) and incompletely represented expression shapes set completeness false
and produce syntax diagnostics. Completeness is independent of provenance:
`COALESCE(t.v, EXCLUDED.v)` and `GREATEST(t.v, EXCLUDED.v)` have complete syntax
and `derived` provenance. Nested calls, literal/placeholder arguments, and
parenthesized/cast roots retain this distinction. Unknown qualifiers and an
`excluded` target-alias collision retain `unresolved` provenance but do not make
represented column syntax incomplete. Wildcard call arguments, subqueries,
roots without typed operand children, and opaque function modifiers (including
FILTER, aggregate ORDER BY, WITHIN GROUP and window clauses) remain incomplete. Derived or unresolved
lineage alone produces no unsupported-syntax diagnostic. Syntax the prepared
parser cannot represent produces the
existing source diagnostic and preserves neighboring valid statements. No raw
parser AST, embedded-language extraction, or consumer policy is exported. All
INSERT contracts are named exported TypeScript types available from both Node
facades. The asynchronous API reuses its prepared token stream and parses each
expression once, including partial-index conflict predicates.

An outer `WITH` on `INSERT ... SELECT` is retained in the source query's CTE
facts. Outer CTEs with `VALUES` or `DEFAULT VALUES`, and overlapping outer and
source-level `WITH` scopes, currently produce incomplete INSERT facts. Signed
numeric constants have literal assignment provenance and fully represented
unary operands; unary expressions over columns remain unresolved but can have
complete syntax. Unary wrappers around binary operands without typed children remain incomplete. Procedural blocks inherit incomplete INSERT facts,
including INSERT occurrences inside conditional branches.

`VALUES` source facts expose rows. Query-level CTEs, ordering, pagination,
locking, and other query modifiers that are absent from that row contract mark
the INSERT incomplete rather than silently discarding their meaning.

### Constraint source spans

`PostgresSqlConstraint.span` is the exact UTF-8 source range for a supported
CREATE TABLE inline or table constraint, ALTER TABLE ADD COLUMN inline
constraint, or ALTER TABLE ADD CONSTRAINT.
Offsets are zero-based bytes, line/column positions are one-based Unicode
characters, and ends are exclusive. A named constraint starts at `CONSTRAINT`;
an unnamed one starts at its constraint keyword (`REFERENCES` for an inline
foreign key). ALTER spans exclude the `ADD` prefix and include `NOT VALID` when
present. Commas, statement delimiters, and surrounding whitespace are excluded;
comments and formatting inside the constraint are retained.

```sql
CREATE TABLE public.children (
  parent_id uuid REFERENCES public.parents(id),
  CONSTRAINT children_check CHECK (parent_id IS NOT NULL)
);
ALTER TABLE public.children
  ADD CONSTRAINT children_parent_fk
  FOREIGN KEY (parent_id) REFERENCES public.parents(id) NOT VALID;
```

The inline foreign key spans `REFERENCES public.parents(id)`, the table CHECK
starts at `CONSTRAINT children_check`, and the ALTER foreign key starts at
`CONSTRAINT children_parent_fk` and ends after `NOT VALID`. Existing statement
spans and identifier/type facts remain available. `constraint.sql` retains its
formatted parser spelling; slice the owning source with `constraint.span` to
preserve original text. Supported procedural and wrapper children use original
source coordinates; literal EXECUTE children use their owning `decodedSql`.

A null span means the prepared tokens cannot prove the constraint's boundaries;
consumers must not infer a location from formatted SQL. Unsupported or malformed
syntax retains its existing diagnostic or incomplete-fact behavior.

### Constraints inside conditional DO bodies

Supported `ALTER TABLE ... ADD CONSTRAINT` occurrences beneath nested `IF`,
`ELSIF`, and `ELSE` branches use the same typed `alterTable.operations` facts as
top-level statements. For example:

```sql
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sample_constraint') THEN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sample_constraint') THEN
      ALTER TABLE sample_child ADD CONSTRAINT sample_constraint
        FOREIGN KEY (parent_id) REFERENCES sample_parent(id) NOT VALID;
    END IF;
  END IF;
END $$;
```

The enclosing `doBlock` and its ordered `conditional.branches[].statements`
retain the source hierarchy. The nested `alterTable` names `sample_child`; its
`addConstraint` operation names the foreign key, local and referenced columns,
referenced relation, and `notValid: true`. Statement spans use original global
UTF-8 byte offsets, so a nested occurrence remains distinguishable from a
top-level statement. Comments, dollar quoting, single-quoted encoding, quoted
or schema-qualified names, and neighboring top-level statements retain their
source boundaries. No PostgreSQL AST or execution policy is returned.

Safely attributed facts can coexist with incomplete procedural coverage.
For example, unsupported `LOCK` statements retain explicit `other` source
occurrences and localized diagnostics instead of hiding a following typed
constraint. Unsupported ALTER operations, incomplete SELECT or INSERT facts,
and unsupported control flow keep the enclosing block incomplete. Inspect its
diagnostics and nested typed facts; these are occurrences, not proof that the
constraint executes, is installed, or is validated.

### Data-modifying CTEs

`query.nestedStatements` exposes INSERT, UPDATE, DELETE and MERGE bodies in
original source order. Each child includes an exact `sql` slice, byte `span`,
`cteId`, `queryScopeId`, `parentScopeId`, typed fields, RETURNING items,
`complete` and `unsupported`. All IDs refer to the containing query report.
An INSERT source query references its scope in that same report; it does not
create a separate query inventory. UPDATE/DELETE/MERGE targets and FROM/USING
inputs reference typed entries in `query.relations`. Unsupported modifiers or
actions retain their typed child and make both the child and query incomplete.
INSERT children also retain existing conflict diagnostics. Source locking clauses
(such as `FOR UPDATE`, including inside nested source queries) have no typed
projection and explicitly make the INSERT child and query incomplete.

```js
const [statement] = (
  await parsePostgresSql({
    sql: "WITH a AS (INSERT INTO target (id) VALUES (1) RETURNING id) SELECT id FROM a",
  })
).statements;
const child = statement.query.nestedStatements[0];
console.log(child.kind, child.cteId, child.returning); // insert, 0, typed items
```

A SELECT CTE remains an ordinary query scope and does not appear in this array.
The CTE `referenced` and `used` flags continue to describe relation references and
SELECT reachability: an unreferenced modifying CTE still appears even when both
flags are false. [PostgreSQL executes data-modifying CTEs](https://www.postgresql.org/docs/current/queries-with.html#QUERIES-WITH-MODIFYING) even without references
or RETURNING, and does not promise an execution order among them; source order
is only a deterministic presentation order. These are syntactic facts, not SQL
semantic validation, so nested modifying CTE syntax can be projected even where
PostgreSQL would reject its execution. No SQL is executed.

Parser rejection is distinct from projection incompleteness: when the shared
parser cannot represent a statement, the source report includes a diagnostic
and no enclosing query facts for that rejected statement. For example, an
ON CONFLICT target predicate inside a CTE currently exceeds that parser's
nested INSERT support. Following valid statements are still collected.

MERGE ON and WHEN conditions are retained as typed predicates and query facts
with clause `other` and non-mandatory context: they choose actions rather than
restrict every affected row. UPDATE and DELETE WHERE conditions retain ordinary
WHERE predicate context.

ON CONFLICT assignment values and action WHERE predicates participate in the
same query's column, equality, subquery and CTE-reference facts. Conflict action
predicates have clause `other` and non-mandatory context because they apply only
to that conditional action. INSERT target aliases resolve through the query's
typed target relation; excluded-column provenance remains in the INSERT payload.
Unsupported assignment expression shapes retain their nested query facts
alongside the
explicit incomplete child.

Write targets bind to physical table facts even when a CTE declares the same
unqualified name. FROM/USING sources retain ordinary lexical CTE lookup.

## Comments, index metadata, and string continuations

Both Node facades and the Rust source API return `{ kind: "comment", comment }`
for supported COMMENT targets. The named `PostgresSqlComment` contract retains
`objectType`, the exact quoted/qualified target identity, nullable comment text,
and routine argument modes, names, and typed signatures. `arguments: []`
preserves an explicit `FUNCTION f()` signature; `null` means no signature was
written. A comment is metadata and does not imply routine invocation. Plain, escape,
Unicode (including `UESCAPE`), and dollar-quoted strings are accepted; bare words
and standalone `USER` targets are rejected. Routine comments inside conditional
DO branches preserve the same signatures.

```sql
COMMENT ON FUNCTION example_function() IS 'documentation';
ALTER INDEX example_parent ATTACH PARTITION example_child;
COMMENT ON TABLE example_table IS
  'First. '
  'Second.';
```

ALTER INDEX ATTACH PARTITION and RENAME TO return `{ kind: "alterIndex", index }`.
`PostgresSqlAlterIndex` preserves the index name, `ifExists`, and a named
`PostgresSqlAlterIndexOperation`: `attachPartition` carries the child index
identity, while `rename` carries the unqualified new name. `IF EXISTS` is supported
for rename and rejected for attach. These are source facts without
catalog lookup, SQL execution, or migration policy.

Plain and escape-prefixed single-quoted strings separated by whitespace containing a newline are
joined in the prepared token inventory, including line-comment continuations.
Continuation segments inherit the first escape-prefixed literal's escape state.
Statement SQL and spans still refer to the original source. The same handling
applies to defaults and INSERT expressions. PostgreSQL does not concatenate
same-line literals or literals separated by a block comment; those spellings
are not normalized. See the [PostgreSQL lexical rules](https://www.postgresql.org/docs/18/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS)
and [scanner continuation grammar](https://github.com/postgres/postgres/blob/REL_18_STABLE/src/backend/parser/scan.l).

Malformed routine defaults, multiple comment targets, missing index targets,
and malformed SQL such as `CREATE TABLE invoices (amount_minor_units BIGINT,,);`
remain diagnostics without successful facts. Recovery preserves the next
independent statement even when a missing value consumes its semicolon.

## Statement wrappers and execution context

EXPLAIN and PREPARE emit `kind: "wrapper"` with a named `PostgresSqlWrapper`.
Its `wrapperKind` is `explain` or `prepare`; `statements` contains ordered child
source occurrences, with original SQL, byte spans, and local ordinals. The
wrapper has its own span, completeness, and diagnostics. Execution is explicit:

- Plain EXPLAIN is `nonExecuting`.
- EXPLAIN ANALYZE is `executesForAnalysis`. The ANALYZE option accepts TRUE/ON/1,
  FALSE/OFF/0, and an omitted value meaning TRUE. Quoted boolean strings accept true/false/on/off, but quoted numeric strings
  remain invalid. Signed integer +1, +0, and -0 have the same boolean meaning;
  other signed numbers remain invalid. Quoted option names preserve exact lowercase identity.
- PREPARE is `nonExecuting`; preparing a statement does not execute it.
- Unsupported or ambiguous EXPLAIN options are `unknown` and incomplete.
  Repeated options use the final value of each name, including ANALYSE as an
  alias of ANALYZE. Every occurrence must still have a valid value. Incompatible
  final ANALYZE/GENERIC_PLAN or WAL/TIMING/SERIALIZE settings fail closed.

Complete EXPLAIN/PREPARE child projection currently covers INSERT and query
facts. Other parsed children retain their source facts but make the wrapper
incomplete; for example, CREATE TABLE AS does not expose its query in the existing
CREATE TABLE contract. SELECT INTO also remains incomplete because target-creation facts are not
represented by the existing query contract. Existing parser compatibility restoration and partial
conflict predicates also apply to supported top-level and atomic-body wrapped children.
Conditional wrappers reuse their enclosing AST's WITH INSERT normalization and
source-query provenance. Partial-index conflict predicates inside procedural
conditional grammar remain unsupported and produce incomplete body diagnostics.
Parse failures
remain typed incomplete wrappers and recover at original statement delimiters.

CREATE FUNCTION retains `kind: "createFunction"` and all existing function
fields, adding `function.wrapper` with `wrapperKind: "functionDeclaration"` and
`execution: "nonExecuting"`. SQL-language BEGIN ATOMIC bodies expose ordered
source occurrences through the same prepared parser. The body is bounded by its
matching END; nested CASE, comments, and string literals retain original spans.
Transaction BEGIN does not introduce a nested atomic body or consume following statements.
Unreserved BEGIN/ATOMIC names and CASE/END labels retain identifier meaning;
function parameters and qualified names do not change the body boundary.
Bare END labels before query continuation tokens also preserve the declaration
boundary; where the native child grammar cannot project them, the declaration
stays explicitly incomplete. An ambiguous END without a preceding child delimiter
remains a recovery boundary rather than inferring unsupported label syntax.
Opaque string bodies and unsupported languages or body forms remain declarations
with incomplete child facts and localized diagnostics. The wrapper's incomplete
status describes its child projection, not the existing function signature.

Execution context belongs to the enclosing hierarchy: a child INSERT inside a
function declaration or PREPARE is a source occurrence, never an executed INSERT
from that declaration. Nested wrappers retain their own syntax classification;
consumers must also honor ancestor execution context. The parser never executes
SQL, interprets function behavior, resolves prepared plans, exports raw parser
ASTs, or applies replay policy. Wrapper projection has a bounded nesting limit.

Supported EXPLAIN options include `SERIALIZE NONE`, `TEXT`, or `BINARY`; omitted
SERIALIZE values mean TEXT. Enabled SERIALIZE, WAL, and TIMING require ANALYZE.
Invalid combinations report unknown execution and incomplete child coverage.
Nested atomic function declarations remain one child declaration occurrence, with
bounded nesting and their own non-executing context.

Conditional IF/ELSE branches retain the same typed EXPLAIN/PREPARE wrappers.
SQL atomic bodies accept an omitted language or an unquoted SQL name; quoted
language identifiers preserve case, so `"sql"` is supported and `"SQL"` is not.
A conflicting AS body followed by BEGIN ATOMIC remains incomplete and retains
its original `bodySql`. PostgreSQL FORMAT options require parentheses; bare
legacy FORMAT forms report unknown execution and incomplete facts.

`EXPLAIN` accepts PostgreSQL's `ANALYSE` alias and `SERIALIZE OFF`. FORMAT and
SERIALIZE identifier values fold to lowercase; quoted string values retain their
case, so unsupported values such as `'JSON'` and `'TEXT'` remain incomplete.
Plain, escape-prefixed, Unicode, and dollar-quoted option strings reuse their
prepared decoded values with the same case-sensitive comparisons.
Conditional wrapper children use the same compatibility restoration as
top-level wrappers, including original generated-column storage modes.

Atomic declarations and conditional wrappers share prepared COMMENT validation,
anchored to statement and wrapper-child positions so query table or alias names
such as `JOIN comment ON ...` retain their query identity,
including routine signatures, Unicode escapes, and string continuations.
Malformed metadata leaves its declaration incomplete and retains independent
following statements. EXPLAIN with a COMMENT child is invalid PostgreSQL and
remains `unknown`/incomplete; PREPARE with that child remains non-executing and
incomplete. Child SQL and spans retain the full original metadata occurrence.

The legacy `EXPLAIN ANALYSE` spelling shares the same prepared token normalization
in top-level statements, conditional branches, and atomic declarations.

Literal PL/pgSQL `EXECUTE` in supported DO bodies exposes `literalExecute` facts
with the exported `PostgresSqlLiteralExecute` payload. Dollar-quoted, ordinary
single-quoted, and `E` escape strings are decoded before parsing their ordered
SQL children through the existing fact pipeline. `literalSpan` and the enclosing
statement span refer to original source; every child and diagnostic span refers
to `decodedSql`, including expression spans. Escapes therefore never produce
fabricated original source coordinates. Malformed nested SQL is diagnostic and
incomplete. Variables, concatenation, format calls, and EXECUTE modifiers remain
unsupported occurrences; source facts imply no execution or replay policy.
Supported IF/ELSIF/ELSE branches, including nested branches, share this literal
projection. SQL CASE expressions and quoted EXECUTE identifiers retain their
ordinary SQL meaning.
