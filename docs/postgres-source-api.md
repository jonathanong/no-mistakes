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
its occurrence list. Typed `IF`, `ELSIF`, and `ELSE` blocks expose a `conditional`
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
