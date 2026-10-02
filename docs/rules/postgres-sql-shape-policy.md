# `postgres-sql-shape-policy`

Ban configured PostgreSQL SQL shapes that the parser can prove. The default
shape is `correlated-exists-set-operation`: a correlated
`EXISTS (SELECT … UNION [ALL] / INTERSECT / EXCEPT …)`. Postgres cannot turn
that set-operation body into a nested loop join, so it rebuilds the whole set
once per outer row. An uncorrelated `EXISTS (… UNION …)` is planned once as an
InitPlan and is not this shape.

The rule consumes dual-source statement facts (`CheckFactPlan.postgres_dml`),
including recoverable SQL returned from builders and fragments appended to a
known SQL builder (a recovered SQL binding or a parameter typed
`SQLStatement`). Arbitrary `.append(...)` receivers are not SQL builders.
Builder recovery honors the same trusted-tag and lexical-shadow rules as
executed SQL. Unresolved raw appended identifiers become a synthetic qualified
outer reference, so a builder that may append an outer column is conservatively
treated as possibly correlated. Identical executed and builder SQL is reported
once. Unparseable or dynamic SQL, including builder fragments, fails closed
unless `unanalyzableSql` is `ignore`.

```yaml
rules:
  - rule: postgres-sql-shape-policy
    scope: repository
    options:
      sqlInclude: ["**/*.sql"]
      bannedShapes:
        - correlated-exists-set-operation
        - not-in-subquery
        - count-for-existence
      unanalyzableSql: fail
```

`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. `not-in-subquery` and
`count-for-existence` are opt-in. Unknown `bannedShapes` values are a
configuration error. `unanalyzableSql` defaults to `fail` (`fail` or `ignore`;
other values are a configuration error).
`importSpecifier` defaults to `@data-stores/psql`; `executorNames` defaults to
`[query, read, write]`.

Counterexample: correlated `EXISTS` wrapping `UNION ALL`.

```sql
SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
```

Fix: keep the set operation uncorrelated, then test membership — for example
`post_id IN (SELECT candidate.post_id FROM ( … UNION ALL … ) candidate)` — or
use a single-arm `EXISTS` without a set operation.

```sql
SELECT 1 FROM posts
WHERE posts.id IN (
  SELECT candidate.post_id FROM (
    SELECT topics.post_id FROM topics
    UNION ALL
    SELECT tags.post_id FROM tags
  ) candidate
);
```

An uncorrelated `EXISTS` around a set operation, including a FROM-less
`SELECT EXISTS (… UNION …) AS alias` scalar probe, is allowed. An inner
placeholder on every arm does not make a correlated wrapping `EXISTS` safe.

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the `EXISTS`, or `no-mistakes-disable-line` on that line, or
`no-mistakes-disable-file` when a whole file is an intentional exception.

## Why and when

Use this rule when correlated `EXISTS (… UNION …)` has caused per-outer-row
set-operation scans, and you want a fail-closed shape ban instead of a
table-specific allowlist.

## What it catches/requires

When `correlated-exists-set-operation` is banned, an `EXISTS` whose subquery
is a set operation is a finding when a qualified `table.column` in that
subquery names a relation that is not local to the subquery's FROM/WITH
(including SELECT-list and HAVING `EXISTS`). Set operations nested inside a
derived-table `FROM` of the `EXISTS` subquery are not this shape.

When `not-in-subquery` is banned, `NOT IN (SELECT …)` and
`NOT (… IN (SELECT …))` are findings. `NOT IN` of a value list and `IN (SELECT …)`
are not. When `count-for-existence` is banned, a built-in `COUNT(*)` compared
with 0 or 1 to test existence is a finding: `> 0`, `>= 1`, `<> 0`, `!= 0`,
`= 0`, `< 1`, `<= 0`, and the mirrored forms. The count may be a scalar
subquery with no `GROUP BY`, or a bare `COUNT(*)` in a SELECT list, WHERE, or
HAVING of a query with no `GROUP BY`.
Grouped `HAVING COUNT(*) > 0` and comparisons with any other number are not
findings. Lowercase quoted built-ins (`"count"` or `pg_catalog."count"`) are
recognized; quoted uppercase or custom-schema functions are excluded.

Correlation is a syntax heuristic: only qualified references count, an
aliased inner relation hides its base name, `schema.table.col` uses the table
component, UNION branches share one local-name set, and nested subquery
scopes are not tracked.

## Options and defaults

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. Also accepted, and off unless listed:
`not-in-subquery`, `count-for-existence`. Unknown `bannedShapes` values are a
configuration error. `unanalyzableSql` defaults to `fail` (`fail` or `ignore`;
other values are a configuration error). `importSpecifier` defaults to
`@data-stores/psql`. `executorNames` defaults to `[query, read, write]`.

## Valid example

```sql
SELECT 1 WHERE EXISTS (
  SELECT 1 FROM topics WHERE id = $1
  UNION ALL
  SELECT 1 FROM topics WHERE id = $1
);
SELECT id FROM accounts a WHERE NOT EXISTS (SELECT 1 FROM bans b WHERE b.account_id = a.id);
SELECT id FROM accounts WHERE id NOT IN (1, 2, 3);
SELECT account_id FROM orders GROUP BY account_id HAVING COUNT(*) > 0;
```

## Counterexample

```sql
SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans);
SELECT COUNT(*) > 0 AS has_orders FROM orders WHERE account_id = $1;
```

## Fix

Rewrite so the set operation is not inside a correlated `EXISTS`: test
`IN (SELECT … FROM (<set-operation>) alias)` or use one restricted subquery.
Replace `NOT IN (SELECT …)` and a `COUNT(*)` compared with 0 or 1 with
`NOT EXISTS (SELECT 1 FROM … WHERE …)` or `EXISTS (SELECT 1 FROM … WHERE …)`.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the finding, or `no-mistakes-disable-line` on that line; use the file
directive only for an intentional exception.

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-idempotent-insert`](postgres-idempotent-insert.md)
covers replay-safe INSERT.

The opted-in `not-in-subquery` shape also checks `HAVING`, aggregate `FILTER`,
and mutation predicates. Nested `NOT` operators are evaluated together so an
even number of negations does not report an allowed `IN` predicate.

`count-for-existence` recognizes transparent casts of scalar `COUNT(*)` values
and the built-in `COUNT(*)` or `pg_catalog.count(*)` aggregate. It deliberately
skips `COUNT(expression)`: that count excludes NULL expressions, so replacing
it with a plain `EXISTS` would change the result when matching rows contain only
NULL values. Schema-qualified custom functions and window counts are excluded.
Findings recommend `EXISTS` for presence checks and `NOT EXISTS` for zero-row
checks, preserving the predicate.
Surrounding `NOT` operators are folded into the count comparison polarity;
use the recommended EXISTS form for the complete negated comparison.
Ungrouped bare counts in `HAVING` are existence probes and are reported.
Grouped bare counts in `HAVING` remain allowed. For example,
`GROUP BY account_id HAVING COUNT(*) > 0` tests each group rather than a scalar
existence probe.

Locations use parsed expression spans, including SQL comments, Unicode names,
`!=` operator spelling, repeated statements, and embedded SQL declaration
lines. Line-specific suppressions therefore apply at the reported expression.
Unanalyzable SQL findings use a target from the enabled `bannedShapes`.

Empty grouping sets (`GROUP BY ()`, including empty `GROUPING SETS`, `ROLLUP`,
and `CUBE`) retain global aggregate semantics, so their count existence probes
are checked. Aggregate FILTER predicates contribute one fact per occurrence.

Embedded findings use the literal's physical source line, including cooked newline
escapes, line continuations, and literals that begin below their declaration.
The Rust facts retain original SQL columns in `SqlSelectFact.not_in_columns` and
`SqlCountExistenceFact.column`; embedded calls expose compact source-line
transitions through `EmbeddedSqlCall.sql_source_positions`.
