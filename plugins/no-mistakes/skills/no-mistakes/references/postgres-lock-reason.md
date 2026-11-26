# PostgreSQL lock reason

Enable `postgres-lock-reason` to require a `lock-reason:` comment on every
configured PostgreSQL row lock, table lock, advisory lock, and configured lock
helper call in production SQL.

The rule is off by default and defaults to `severity: advisory` (non-failing).
Raise a rule application to `severity: error` once existing production locks
are documented. Exclude test files and test helpers via `include` / `exclude`
or the referenced project's `postgres` scope.

Each lock site needs a comment in the same SQL statement or on contiguous lines
directly above the statement or executor/helper call containing `lock-reason:`
with two non-empty labelled parts:

```sql
-- lock-reason: prevents: double refund; writer: refund worker retry
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;
```

- `prevents:` names the race or invariant failure the lock prevents.
- `writer:` names the competing writer the lock serializes against.
- `no-mistakes: deadlock-safe -- ...` does not satisfy `postgres-lock-reason`.

Configure executor selection (`importSpecifier`, `executorNames`,
`executorFactoryNames`, `executorTypeNames`, `executorOptionProperties`,
`executorOptions`, `trustedSqlTags`), `sqlInclude`, `helpers`, and
`unanalyzable` (`report`, `advise`, or `ignore`; default `advise`). Unresolved
SQL, parse failures, stored routine bodies, truncated alternatives, and
unresolved executor receivers never pass as reasoned even when a comment is
nearby.

When `postgres-lock-reason` shares executor configuration with
`postgres-lock-inventory`, both rules reuse one prepared SQL and lock-site
projection. Ordinary `check --format json`, `check()`, and
`analyzeProject({ reports: [{ type: "check" }] })` expose findings and
advisories.

See the [rule contract](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/postgres-lock-reason.md).
