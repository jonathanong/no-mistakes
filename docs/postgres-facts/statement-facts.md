<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Statement facts

`extract_sql_statement_facts(sql)` parses PostgreSQL SQL (leniently) and
returns typed INSERT / SELECT / CREATE TRIGGER facts without keeping the
sqlparser AST:

For SQL returned by `extract_embedded_sql_from_source`, pass its
`EmbeddedSqlCall` to `extract_sql_statement_facts_for_embedded_call(&call)`.
This composition API preserves the exact positions of generated interpolation
markers for sweep and cursor-bind facts, bounded pins, LIMIT and EXISTS
restrictions, value forms, mutation predicates, and relation-restriction facts.
A user-written identifier such as `sql_placeholder_1` remains a column while
the generated marker is treated as a bind. `SqlColumnUseFact` still uses the
legacy marker-spelling heuristic; exact-position handling there is tracked in
[issue #1391](https://github.com/jonathanong/no-mistakes/issues/1391).
It returns `None` when the call has no recovered SQL. This singular helper
preserves the legacy `sql_text` contract; it does not select an alternative
from `variants`. Fact line and column locations are relative to the recovered
SQL text; a `Dynamic` call's legacy text can contain only a verified leading
fragment. Prepared rule analysis uses complete variants and rebases their
locations through each version's source map. Use `extract_sql_statement_facts(sql)` for
standalone SQL without embedded interpolation provenance.

The Rust-only `SqlStatementFileFacts.variant_locations` sidecar records typed
occurrence positions (`SqlFactSite` / `SqlFactPosition`), the original executor
call, and prepared lock/conflict metadata for complete alternatives. It is
absent for legacy sources and is not a Node JSON output contract. This lets
rules share the prepared SQL AST projection, preserve executor-line suppressions,
and map branch findings without parsing the SQL again.

- executed `INSERT` (EXPLAIN without ANALYZE is skipped; PREPARE inner
  statements are treated as executed; CREATE FUNCTION/PROCEDURE bodies are not)
- `ON CONFLICT` action, arbiter (columns / named constraint / unknown), SET
  assignment forms (literal, EXCLUDED, self-ref, COALESCE/GREATEST/LEAST,
  placeholder, volatile, subquery, other), conjunctive WHERE proofs
  (`IS DISTINCT FROM EXCLUDED`, `IS NULL AND EXCLUDED IS NOT NULL`), and
  INSERT column value forms from `VALUES` / `SELECT` (or MySQL-style `SET`).
  Wildcards, omitted columns, `DEFAULT VALUES`, and set operations yield no
  stable form; `DEFAULT`, `CURRENT_TIMESTAMP`, `'now'`, `N'now'`, and `OVERRIDING USER
  VALUE` are unstable
- `INSERT…SELECT` guarded by a conjunctive `WHERE NOT EXISTS`
- Physical TABLE set-operation arms in UPDATE and DELETE subqueries reuse the
  prepared token cursor, including assignments, FROM/USING, WHERE, and RETURNING.
  Visible CTE aliases stay local, quoted names retain identity, and EXISTS arms
  omit explicit-column stars. These Select facts share the CLI and async Node
  rule projections without changing mutation predicates or bound facts.
- SELECT FROM/JOIN relation names, predicate SQL, and `EXISTS` set-operation
  facts (`restricted` when every arm has a placeholder or literal bound;
  `correlated` when a qualified identifier is outside the subquery FROM/WITH;
  `column` is the `EXISTS` keyword in the original SQL)
- one `SqlRelationPredicateFact` per base-table instance of each SELECT
  (`relations`), plus grouped `updates` and `deletes` (one group per
  statement). Each fact records the table, optional alias, source line,
  `constrained_columns` (qualified, or unqualified when that statement has a
  single FROM item), and `unqualified_columns` (bare names from a
  multi-relation FROM that a schema catalog may still prove). A column is
  constrained by a top-level AND conjunct that is an equality, comparison,
  `IN`, `= ANY`, or `BETWEEN`; `OR` keeps a column only when every branch
  constrains it. `in_insert_select` marks SELECT facts that belong to
  `INSERT … SELECT`, including selects nested in that query.
  `not_in_subqueries` records lines of `NOT IN (SELECT …)` and
  `NOT (… IN (SELECT …))`. `count_existence_checks` records each `COUNT(*)`
  compared with 0 or 1 as a `SqlCountExistenceFact`: the comparison's line,
  its SQL column, and `negated` (true when the comparison tests zero rows).
  `postgres-sql-shape-policy` reads both when those shapes are banned.
  `star_projections` records each bare `*` (one fact per base FROM relation) and
  each `alias.*` (that alias's base relation). Stars inside `EXISTS` / `NOT EXISTS`,
  `COUNT(*)`, CTEs, and derived-table aliases are omitted. Recursive WITH exposes
  every sibling CTE alias within every body; non-recursive bodies see only earlier
  aliases. Schema-qualified names still identify physical relations. A star passed to a
  function records `within_function` so `postgres-explicit-columns` can apply
  `allowWholeRowFunctions` without parsing SQL again.
  `returning_stars` on the file facts records `RETURNING *` / `RETURNING t.*`
  for INSERT, UPDATE, and DELETE.
  `column_uses` records bare column references from comparisons in WHERE and
  JOIN ON, and from ORDER BY. An empty `table` means the name was unqualified
  across more than one base relation. `postgres-generated-column-predicates`
  reads these facts together with migration schema facts. Recovered embedded binds
  are omitted using their SQL-local source positions before resolving ORDER BY
  projection aliases. Literal and quoted marker-shaped identifiers remain ordinary
  columns, including in standalone SQL.
- `CREATE TRIGGER` table, function, period, row/statement, and events

Unparseable files set `parse_failed` and count quote-masked `INSERT INTO`
keywords so multi-INSERT fragments fail closed. A top-level conjunctive
`WHERE NOT EXISTS` / `AND NOT EXISTS` (paren-depth zero) is recorded even
when the AST is missing.

`postgres-required-predicates` consumes the relation-predicate facts and, when
`partitionKeys` is `require`, the schema catalog's relation kind, columns, and
partition key. `postgres-sql-shape-policy` and `postgres-idempotent-insert`
consume the same statement facts.

[PostgreSQL fact sources](../postgres-facts.md)
