# `postgres-sql-shape-policy`

Ban configured PostgreSQL SQL shapes that the parser can prove. The default
shape is `correlated-exists-set-operation`: `EXISTS (SELECT … UNION [ALL] …)`
without an inner placeholder or literal restriction on every arm. That form
often correlates over an unbound relation and cannot be proven safe.

The rule consumes dual-source statement facts (`CheckFactPlan.postgres_dml`).
Unparseable or dynamic SQL fails closed unless `unanalyzableSql` is `ignore`.

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

Counterexample: `EXISTS` wrapping `UNION ALL` with no inner restriction.

```sql
SELECT 1 WHERE EXISTS (
  SELECT 1 FROM topics
  UNION ALL
  SELECT 1 FROM topics
);
```

Fix: restrict each set-operation arm (placeholder or literal) or rewrite
without `EXISTS` around a set operation.

```sql
SELECT 1 WHERE EXISTS (
  SELECT 1 FROM topics WHERE id = $1
  UNION ALL
  SELECT 1 FROM topics WHERE id = $1
);
```

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule when correlated `EXISTS (… UNION …)` over an unbound relation
has caused full scans or correctness holes, and you want a fail-closed shape
ban instead of a table-specific allowlist.

## What it catches/requires

When `correlated-exists-set-operation` is banned, an `EXISTS` whose subquery
is a set operation is a finding unless every arm has an inner restriction
(placeholder, numeric/string literal, or `sql_placeholder_*`).

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
SELECT 1 WHERE EXISTS (
  SELECT 1 FROM topics
  UNION ALL
  SELECT 1 FROM topics
);
```

## Fix

Put a placeholder or literal restriction on every UNION/EXCEPT/INTERSECT arm,
or replace the set operation with a single restricted subquery.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` or
`no-mistakes-disable-line`; use the file directive only for an intentional
unrestricted existence check.

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-idempotent-insert`](postgres-idempotent-insert.md)
covers replay-safe INSERT.
