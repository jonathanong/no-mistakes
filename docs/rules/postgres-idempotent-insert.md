# `postgres-idempotent-insert`

Require executed `INSERT` statements to be replay-safe: `ON CONFLICT DO
NOTHING`, a conjunctive `WHERE NOT EXISTS`, or `ON CONFLICT DO UPDATE` that
converges. Optional follow-on proofs (volatility, arbiter identity, triggers,
generated-arbiter sources) default on and can be disabled.

The rule consumes dual-source statement facts plus schema facts for generated
columns and `CREATE TRIGGER`. Unparseable fragments with more than one
`INSERT` fail closed. Dynamic embedded SQL fails closed unless
`unanalyzableSql` is `ignore`.

```yaml
rules:
  - rule: postgres-idempotent-insert
    scope: repository
    options:
      sqlInclude: ["**/*.sql"]
      scanEmbedded: true
      checkConvergence: true
      checkVolatility: true
      checkArbiter: true
      checkTriggers: true
      checkGenerated: true
      replaySafeTriggerFunctions: []
      triggerWrittenColumns: {}
      unanalyzableSql: fail
```

`sqlInclude` defaults to `**/*.sql`. `scanEmbedded` and every `check*` flag
default to `true`. `replaySafeTriggerFunctions` defaults to `[]`.
`triggerWrittenColumns` defaults to `{}`. `unanalyzableSql` defaults to `fail`
(`fail` or `ignore`; other values are a configuration error).
`importSpecifier` defaults to `@data-stores/psql`; `executorNames` defaults to
`[query, read, write]`.

Counterexample: a bare insert.

```sql
INSERT INTO items (id) VALUES (1);
```

Fix: add `ON CONFLICT DO NOTHING`, a conjunctive `NOT EXISTS` guard, or a
convergent `DO UPDATE`.

```sql
INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
```

Use `no-mistakes-disable-next-line postgres-idempotent-insert` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule for SQL that workers, retries, or notification replay may run
more than once. Identity-preserving `INSERT` prevents duplicate rows and
non-convergent `now()` / `gen_random_uuid()` rewrites.

## What it catches/requires

Executed inserts must include `ON CONFLICT` or a top-level conjunctive
`WHERE NOT EXISTS`. `NOT EXISTS` is a sequential-replay heuristic: under
`READ COMMITTED`, concurrent statements can both pass the guard, so prefer
`ON CONFLICT` with a unique arbiter when writers can overlap. `DO UPDATE` SET
lists must be convergent; volatile functions may only follow a self-reference
in `COALESCE`; arbiter columns must stay `EXCLUDED` or unchanged; triggers that
re-fire on replay are findings unless allowlisted; allowlisted triggers that
write generated-arbiter source columns still fail.

## Options and defaults

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `scanEmbedded` defaults to `true`.
`checkConvergence`, `checkVolatility`, `checkArbiter`, `checkTriggers`, and
`checkGenerated` default to `true`. `replaySafeTriggerFunctions` defaults to
`[]`. `triggerWrittenColumns` maps function names to columns those functions
write (default `{}`). `unanalyzableSql` defaults to `fail` (`fail` or
`ignore`; other values are a configuration error).
`importSpecifier` defaults to `@data-stores/psql`. `executorNames` defaults to
`[query, read, write]`.

## Valid example

```sql
INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
```

## Counterexample

```sql
INSERT INTO items (id, seen) VALUES (1, now())
ON CONFLICT (id) DO UPDATE SET seen = now();
```

## Fix

Use `DO NOTHING`, `WHERE NOT EXISTS`, `SET col = EXCLUDED.col`, or
`COALESCE(table.col, now())` so replay is an identity.

## Suppression

Use `no-mistakes-disable-next-line postgres-idempotent-insert` or
`no-mistakes-disable-line`; use the file directive only for an intentional
non-idempotent load.

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-sql-shape-policy`](postgres-sql-shape-policy.md)
bans unsafe `EXISTS` set operations;
[`postgres-no-generated-column-writes`](postgres-no-generated-column-writes.md)
bans writing generated columns in DML.
