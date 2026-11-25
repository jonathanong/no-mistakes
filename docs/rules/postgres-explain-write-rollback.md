# `postgres-explain-write-rollback`

## Why and when

`EXPLAIN ANALYZE` executes the statement it measures. A script run with
autocommit can persist a write, as can a script that explicitly commits its
transaction. Use this opt-in rule for standalone PostgreSQL scripts where the
transaction boundaries are visible in one file.

## Options and defaults

The rule is disabled until configured. Set `sqlInclude` to a nonempty list of
SQL globs for standalone scripts that run in order on one connection with
autocommit enabled and no caller-owned transaction. The rule's `include`,
`exclude`, and target scope can narrow that selection.

`options.unanalyzable` (`report`, `advise`, or `ignore`; default `advise`)
controls scripts that cannot be parsed completely, `RELEASE SAVEPOINT` or
`ROLLBACK TO SAVEPOINT` targeting an unknown savepoint, and dynamic read-only
`SET` or `set_config(...)` arguments.

```yaml
rules:
  - rule: postgres-explain-write-rollback
    scope: repository
    options:
      sqlInclude: [benchmarks/*.sql]
      unanalyzable: advise
```

## What it catches/requires

The rule reports an executing `EXPLAIN ANALYZE` or `EXPLAIN (ANALYZE TRUE)`
of a direct `INSERT`, `UPDATE`, `DELETE`, or `MERGE` when it runs under
autocommit or its transaction ends in `COMMIT`. The finding points to the
outer `EXPLAIN` keyword. Plan-only EXPLAIN and read-only SELECT plans pass.

## Valid example

```sql
-- benchmarks/update.sql
BEGIN;
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
ROLLBACK;
```

## Counterexample

```sql
-- benchmarks/update.sql
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
```

## Fix

Run the write plan inside a transaction ending with `ROLLBACK`, or remove
`ANALYZE` to get the plan without executing the write.

## Suppression

Use `-- no-mistakes-disable-file postgres-explain-write-rollback` for a reviewed
file exception. Use `-- no-mistakes-disable-line postgres-explain-write-rollback`
on the outer `EXPLAIN` line or
`-- no-mistakes-disable-next-line postgres-explain-write-rollback` immediately
above it for one plan.

## Analysis

The rule reuses the check request's parsed SQL and tracks transaction state,
nested savepoints (`SAVEPOINT`, `RELEASE SAVEPOINT`, and
`ROLLBACK TO SAVEPOINT`), `SET` / `SET TRANSACTION` /
`SET SESSION CHARACTERISTICS` read-only settings, and `set_config(...)` calls
for `default_transaction_read_only` and `transaction_read_only` respecting
`is_local` scoping. `RELEASE SAVEPOINT` keeps writes pending for the enclosing
transaction; only `ROLLBACK` or `ROLLBACK TO` an enclosing savepoint discards
them. Files that cannot be parsed completely, unknown savepoint targets, and
dynamic read-only settings are routed through `options.unanalyzable` instead of
passing silently. Embedded SQL, prepared statements, and indirect writes such
as function bodies or modifying CTEs are outside its scope.

See the [PostgreSQL EXPLAIN documentation](https://www.postgresql.org/docs/current/sql-explain.html).

## Related rules

- [`postgres-bounded-statements`](postgres-bounded-statements.md) checks
  how many rows supported statements can read or change.
- [`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) checks
  configured statement kinds and settings in supported SQL sources.
