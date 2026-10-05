# Explicit PostgreSQL executors

The next minor release removes the consumer-specific embedded SQL import
default. This is a breaking configuration change for every PostgreSQL rule
that scans TypeScript or JavaScript executor calls.

To migrate, set `importSpecifier` explicitly to your database module:

```yaml
options:
  importSpecifier: "@example/db"
  executorNames: [query, read, write]
```

A module with no `executorNames` uses `query`, `read`, and `write`. Without a
module, explicitly configured executor names match named imports from any
module. Native SQL and SQL-builder fragment policies retain their existing
scope.

## Selecting no executor calls is an error

A rule that sets neither `importSpecifier` nor `executorNames` would silently
scan zero executor calls and report nothing. `no-mistakes check` therefore
rejects that configuration with a non-zero exit and names the rule and the fix:

```text
error: postgres-lock-ordering option importSpecifier: set importSpecifier (or executorNames) to select executor calls; set executorNames: [] to scan only SQL files and native SQL (see docs/migrations/explicit-postgres-executors.md)
```

A blank `importSpecifier` counts as omitted. To keep a rule on SQL files and
native SQL only, opt out explicitly:

```yaml
options:
  executorNames: []
```

An explicit empty `executorNames` with no module selects no executor calls
(including `.query` members). With a module, an omitted or empty
`executorNames` still uses `query`, `read`, and `write`.
`postgres-idempotent-insert` with `scanEmbedded: false` does not judge executor
calls, so it needs neither option.

This applies to bounded statements, explicit columns, generated-column
predicates and writes, required predicates, SQL shape policy, OFFSET,
conflict ordering, lock ordering, idempotent inserts, and query annotation.

## ESLint

The ESLint PostgreSQL runtime rules (`postgres-no-manual-transaction` and
`postgres-no-unbounded-query-fanout`) use the same explicit executor
configuration. They throw the same configuration error when neither
`importSpecifier` nor `executorNames` is set, and accept `executorNames: []` to
select no executor calls.
