# `postgres-required-predicates`

Require configured WHERE/JOIN predicates whenever a query reads a named
PostgreSQL relation. Use this for tables that must always be filtered (for
example a `parent_id IS NOT NULL` bound) without hardcoding those table names
in the checker.

The rule consumes dual-source statement facts (`CheckFactPlan.postgres_dml`):
matching `.sql` files plus statically recoverable embedded executor SQL. Dynamic
or unparseable SQL fails closed unless `unanalyzableSql` is `ignore`.

```yaml
rules:
  - rule: postgres-required-predicates
    scope: repository
    options:
      sqlInclude: ["**/*.sql"]
      relations:
        - table: topics
          require:
            - parent_id IS NOT NULL
      unanalyzableSql: fail
```

`sqlInclude` defaults to `**/*.sql`. `relations` defaults to empty (no
findings). `unanalyzableSql` defaults to `fail` (`fail` or `ignore`; other
values are a configuration error). `importSpecifier` defaults to
`@data-stores/psql`; `executorNames` defaults to `[query, read, write]`.

Counterexample: `SELECT id FROM topics WHERE id = $1` when `topics` requires
`parent_id IS NOT NULL`.

```sql
SELECT id FROM topics WHERE id = $1;
```

Fix: include every required predicate, typically AND-ed with the rest of the
WHERE/JOIN clause.

```sql
SELECT id FROM topics WHERE parent_id IS NOT NULL AND id = $1;
```

Use `no-mistakes-disable-next-line postgres-required-predicates` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule when some relations are unsafe to scan unbound (deleted rows,
unpublished rows, or a required parent key) and every SELECT/JOIN must carry
those predicates.

## What it catches/requires

SELECT (and INSERT…SELECT) statements whose FROM/JOIN list names a configured
`table` must include each `require` predicate string in the WHERE/JOIN SQL,
compared case-insensitively after whitespace normalization.

## Options and defaults

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `relations` defaults to `[]`.
`unanalyzableSql` defaults to `fail` (`fail` or `ignore`; other values are a
configuration error).
`importSpecifier` defaults to `@data-stores/psql`. `executorNames` defaults to
`[query, read, write]`.

## Valid example

```sql
SELECT id FROM topics WHERE parent_id IS NOT NULL AND id = $1;
```

## Counterexample

```sql
SELECT id FROM topics WHERE id = $1;
```

## Fix

Add the required predicate to WHERE or JOIN ON for that relation.

## Suppression

Use `no-mistakes-disable-next-line postgres-required-predicates` or
`no-mistakes-disable-line`; use the file directive only for an intentional
unfiltered reporting query.

## Related rules

[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) bans correlated
`EXISTS` set operations; [`postgres-idempotent-insert`](postgres-idempotent-insert.md) covers
replay-safe INSERT.
