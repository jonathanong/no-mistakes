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
module. Omitting both options is a configuration error in the next minor release.
To intentionally select no executor calls, omit `importSpecifier` and write
`executorNames: []` explicitly. Native SQL and SQL-builder fragment policies
retain their existing scope. This prevents a CI configuration that has not migrated
from silently passing while skipping executor checks.

This applies to bounded statements, explicit columns, generated-column
predicates and writes, required predicates, SQL shape policy, OFFSET,
conflict ordering, lock ordering, idempotent inserts, and query annotation.

The ESLint PostgreSQL runtime rules use the same explicit executor configuration.
