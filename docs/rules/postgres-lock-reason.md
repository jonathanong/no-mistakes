<!-- cspell:ignore xact -->

# `postgres-lock-reason`

Opt-in check that requires a `lock-reason:` comment on every configured
PostgreSQL row lock, table lock, advisory lock, and configured lock helper call
in production SQL.

## Why and when

Row, table, and advisory locks serialize concurrent writers and can stall
unrelated requests when misused or left behind after a refactor. Requiring a
short `lock-reason:` comment beside each lock forces the author to state both
the race outcome the lock prevents and the competing writer it serializes
against, so reviewers can judge whether the lock is necessary or replaceable
with a guarded write or `SERIALIZABLE` retry.

The rule is off by default and defaults to `severity: advisory`, so enabling it
reports missing or incomplete reasons without failing `check`. Set
`severity: error` on a rule application once existing production locks are
documented.

## Options and defaults

The rule is disabled until configured. Exclude tests and test helpers through
the rule's `include` / `exclude` filters or the referenced project's `postgres`
scope (never inferred from hardcoded test globs).

- `severity`: defaults to `advisory` (non-failing). Set `severity: error` to
  fail `check` on missing or incomplete `lock-reason:` comments.
- Executor selection uses the standard PostgreSQL options: `importSpecifier`,
  `executorNames` (use `[]` to scan SQL files only), `extraExecutorNames`,
  `excludeExecutorNames`, `executorFactoryNames`, `executorTypeNames`,
  `executorOptionProperties`, `executorOptions`, and `trustedSqlTags`.
- `sqlInclude`: SQL file globs for native `.sql` files.
- `helpers`: configured TypeScript or SQL helper function names that acquire
  locks (case-sensitive in TypeScript; unquoted SQL names fold to lowercase).
- `unanalyzable` (`report`, `advise`, or `ignore`; default `advise`): policy
  for dynamic executor SQL, unparseable SQL or TypeScript, stored routine
  bodies (`CREATE FUNCTION` / `CREATE PROCEDURE`), truncated SQL alternatives,
  and unresolved executor receivers. Unresolved SQL and helpers are never
  treated as reasoned, even when a `lock-reason:` comment appears nearby.
  Legacy `unanalyzableSql` is also accepted (`fail` maps to `report`).
- `reportUnmatchedExecutorNames`: report configured `executorFactoryNames` or
  `executorTypeNames` not imported in any scanned file.

Invalid options (unknown keys, empty `helpers` entries, invalid globs, or
invalid `unanalyzable` values) fail at config load with exit code 2.

```yaml
rules:
  - rule: postgres-lock-reason
    scope: repository
    include: ["src/**/*.ts", "db/**/*.sql"]
    exclude: ["**/*.test.ts"]
    options:
      importSpecifier: "@example/db"
      executorNames: [query]
      executorFactoryNames: [beginTransaction]
      helpers: [reserveJob]
      sqlInclude: ["db/**/*.sql"]
```

## What it catches/requires

The rule checks every lock site produced by the shared PostgreSQL lock
inventory projection:

- Row locks: `FOR UPDATE`, `FOR NO KEY UPDATE`, `FOR SHARE`, and
  `FOR KEY SHARE`, including `OF <table>`, `NOWAIT`, and `SKIP LOCKED` forms.
- Table locks: `LOCK TABLE` (including `NOWAIT`).
- Advisory locks: recognized `pg_advisory_*` and `pg_try_advisory_*` calls.
- Configured helpers listed in `options.helpers` (TypeScript and SQL calls).

Each lock site passes when a comment in the same statement or on the line
directly above the SQL statement or executor/helper call contains
`lock-reason:` with two non-empty labelled parts:

- `prevents:` — the race or invariant failure the lock prevents.
- `writer:` — the competing writer the lock serializes against.

Both `--` / `/* ... */` SQL comments and `//` / `/* ... */` TypeScript comments
are recognized; string literals are ignored. Contiguous comment lines directly
above the statement or call (with no blank line or code line between them) are
checked together, so `lock-reason:` may be stacked with other comments or
wrapped across two comment lines.

A `lock-reason:` comment missing `prevents:`, missing `writer:`, or missing
both is reported with a diagnostic naming the missing part.
`no-mistakes: deadlock-safe -- ...` comments explain lock ordering, not why the
lock exists, and do not satisfy this rule.

## Valid example

```sql
-- lock-reason: prevents: double refund; writer: refund worker retry
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

-- lock-reason: prevents: overlapping payout batch; writer: scheduled payout cron
SELECT pg_advisory_xact_lock($1);
```

```ts
import { query } from "@example/db";

export async function claimRefund(id: number) {
  // lock-reason: prevents: double refund; writer: refund worker retry
  await query("SELECT id FROM refunds WHERE id = $1 FOR UPDATE", [id]);
}
```

## Counterexample

```sql
-- Missing lock-reason:
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

-- deadlock-safe does not satisfy lock-reason:
-- no-mistakes: deadlock-safe -- ordered by account_id
SELECT id FROM accounts WHERE id = $1 FOR UPDATE;

-- Missing writer:
-- lock-reason: prevents: double refund
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

-- Advisory lock without lock-reason:
SELECT pg_advisory_lock(42);
```

## Fix

If the lock is unnecessary, replace it with an atomic guarded write or
`SERIALIZABLE` retry. Otherwise add a `lock-reason:` comment directly above the
SQL statement or executor/helper call (or inside the same SQL statement) naming
both `prevents:` and `writer:`:

```sql
-- lock-reason: prevents: double refund; writer: refund worker retry
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;
```

## Suppression

Standard `no-mistakes-disable-file postgres-lock-reason`,
`no-mistakes-disable-line postgres-lock-reason`, and
`no-mistakes-disable-next-line postgres-lock-reason` directives suppress
findings and advisories on reviewed exceptions.

## Analysis

The rule reuses the prepared `postgres-lock-inventory` statement projection and
`EmbeddedSqlOptions` profile with no cross-function reachability pass and no
`effort` mode. When `postgres-lock-inventory` and `postgres-lock-reason` share
the same executor configuration (for example through a project's `postgres`
block), `no-mistakes` runs the SQL extraction and lock-site projection once for
both rules and performs only a local comment lookup on files that contain lock
sites.

## Related rules

- [`postgres-lock-inventory`](postgres-lock-inventory.md) tracks the lock
  surface against a committed baseline and shares the same lock-site projection.
- [`postgres-lock-written-rows-only`](postgres-lock-written-rows-only.md)
  checks row-lock targets and transaction ownership.
- [`postgres-lock-ordering`](postgres-lock-ordering.md) checks acquisition order
  and uses `no-mistakes: deadlock-safe` annotations.
