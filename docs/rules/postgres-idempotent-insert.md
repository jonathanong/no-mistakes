# `postgres-idempotent-insert`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Require executed `INSERT` statements to be replay-safe: `ON CONFLICT DO
NOTHING`, a conjunctive `WHERE NOT EXISTS`, or `ON CONFLICT DO UPDATE` that
converges. Optional follow-on proofs (volatility, arbiter identity, triggers,
generated-arbiter sources) default on and can be disabled.

The rule consumes dual-source statement facts plus schema facts for generated
columns and `CREATE TRIGGER`. Unparseable fragments with more than one
`INSERT` fail closed. A recovered `INSERT` keeps expression conflict-key
dependencies when a malformed neighboring statement forces SQL recovery.
Each complete
[SQL alternative](../postgres-facts.md#finite-sql-alternatives) of a branching
executor call is checked independently, up to 16 versions. Choosing between
`ON CONFLICT DO NOTHING` and a bare INSERT still reports the bare INSERT; a
replay-safe path does not cover an unsafe sibling. Findings retain branch
locations for suppressions. Opaque or over-cap embedded SQL fails closed unless
`unanalyzableSql` is `ignore` or `advise`.

```yaml
rules:
  - rule: postgres-idempotent-insert
    scope: repository
    options:
      importSpecifier: "@example/db"
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
(`fail`, `ignore`, or `advise`; other values are a configuration error).
`importSpecifier` has no default; `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

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
write generated-arbiter source columns still fail. A conjunctive `DO UPDATE`
`WHERE` that proves any one assigned `EXCLUDED` column (`IS DISTINCT FROM
EXCLUDED` or `IS NULL AND EXCLUDED IS NOT NULL`) is a no-op for `AFTER UPDATE`
row triggers, because Postgres skips the whole update when `WHERE` is false.
A proof column listed in `triggerWrittenColumns` for any applicable trigger
still fails: another function on the same update, a `BEFORE` / `INSTEAD OF` /
`AFTER INSERT` function, or a statement-level UPDATE function can rewrite the
column so the next replay sees a true `WHERE`. The proven INSERT value must
also be stable (literal, null, placeholder, or a `COALESCE`/`GREATEST`/`LEAST`
of those). Volatile INSERT values (`now()`, `CURRENT_TIMESTAMP`), relative
datetime literals (`'now'`, `N'now'`, `U&'now'`), `DEFAULT`, `OVERRIDING USER VALUE`,
omitted columns, `SELECT *`, SELECT column references, and sources without a
column list still fail. Disjunctive `WHERE` and constant `SET` plus a matching
`WHERE` still fail.

Expression conflict targets also protect every column that the expression
references. For example, `ON CONFLICT (lower(email)) DO UPDATE SET email = 'b'`
is a finding; `email = email` and `email = EXCLUDED.email` are accepted. Mixed
targets such as `(email, lower(name))` check both dependencies. The rule proves
dependencies for identifiers (including qualified and quoted names), nested
expressions, casts, unary and binary operations, and ordinary positional
function arguments. Operator classes and other expression forms, including
`CASE` and subqueries, cannot be proven complete. A dependency of a single-column
expression still passes when assigned its matching `EXCLUDED` value or itself.
For an expression combining columns, such as `(a + b)`, every dependency must
be assigned its matching `EXCLUDED` value, or the expression must remain
unchanged. Assigning only `a = EXCLUDED.a` could move the conflict key and make
the next replay insert a second row. Separate target expressions such as
`(a, lower(b))` remain independent. Other non-self assignments report an
explicit proof failure. This conservative result may include an unrelated assignment;
simplify the expression or use a rule suppression when you have verified its
safety.

## Options and defaults

See [options and executor configuration](postgres-idempotent-insert-options.md).

## Valid example

```sql
INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
```

A conjunctive `WHERE` on any one assigned `EXCLUDED` column is also enough
for `AFTER UPDATE` row triggers, even when the trigger watches a different
assigned column:

```sql
INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
WHERE items.b IS DISTINCT FROM EXCLUDED.b;
```

## Counterexample

```sql
INSERT INTO items (id, seen) VALUES (1, now())
ON CONFLICT (id) DO UPDATE SET seen = now();
```

## Fix

Use `DO NOTHING`, `WHERE NOT EXISTS`, `SET col = EXCLUDED.col`, or
`COALESCE(table.col, now())` so replay is an identity.

## Analysis

Raw SQL parse failures retain each failing statement's first code token. Separate
statements produce separate findings, and a `no-mistakes-disable-next-line`
directive immediately above one statement suppresses that failure independently.
Successfully parsed sibling statements remain analyzable.
Only failed statements containing an `INSERT` produce replay-safety uncertainty;
a malformed non-INSERT sibling does not make a valid INSERT unsafe.

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. At max it evaluates executor calls in TS/JS (up to 16 finite SQL alternatives per call) unless `scanEmbedded: false`, and SQL files selected by `sqlInclude`.

### Unanalyzable inputs

Valid PostgreSQL can still be unsupported by this analyzer. When it cannot
recover statements such as `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION`,
or `REFRESH MATERIALIZED VIEW`, it reports them under
[`options.unanalyzable`](../migrations/postgres-schema-unanalyzable.md);
the diagnostic does not mean PostgreSQL would reject the statement.

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), inputs this rule cannot prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove: executed `INSERT` calls with no complete recovered alternative or over the cap, and unparseable SQL. They follow `options.unanalyzable`:
  - `report` (default): fails as a finding, e.g. `executed INSERT SQL is not statically recoverable`.
  - `advise`: a non-failing advisory with the same reason.
  - `ignore`: omitted.

  Definite violations remain findings under every policy. The legacy `unanalyzableSql: fail` spelling means `report`; `advise` and `ignore` keep their names.

SQL schema statements selected by `sqlInclude` also retain diagnostics when the
lenient parser cannot parse or recover them. Each diagnostic identifies the file,
statement line, and statement head, and follows the same `options.unanalyzable`
policy. Valid sibling statements remain available to the catalog. A real
`DISCARD ALL` statement does not trigger this recovery diagnostic. Existing file
and line suppressions apply; rewrite unsupported syntax into a supported static
statement or suppress the specific diagnostic with a reason.

## Suppression

Use `no-mistakes-disable-next-line postgres-idempotent-insert` or
`no-mistakes-disable-line`; use the file directive only for an intentional
non-idempotent load.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-sql-shape-policy`](postgres-sql-shape-policy.md)
bans correlated `EXISTS` set operations;
[`postgres-no-generated-column-writes`](postgres-no-generated-column-writes.md)
bans writing generated columns in DML.
