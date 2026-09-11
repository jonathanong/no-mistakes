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
      unanalyzableSql: fail
```

`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. Unknown `bannedShapes` values are a
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

Correlation is a syntax heuristic: only qualified references count, an
aliased inner relation hides its base name, `schema.table.col` uses the table
component, UNION branches share one local-name set, and nested subquery
scopes are not tracked.

## Options and defaults

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. Unknown `bannedShapes` values are a
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
```

## Counterexample

```sql
SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
```

## Fix

Rewrite so the set operation is not inside a correlated `EXISTS`: test
`IN (SELECT … FROM (<set-operation>) alias)` or use one restricted subquery.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the `EXISTS`, or `no-mistakes-disable-line` on that line; use the file
directive only for an intentional correlated existence check.

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-idempotent-insert`](postgres-idempotent-insert.md)
covers replay-safe INSERT.
