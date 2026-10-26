<!-- cspell:ignore uuidv -->

# postgres-sql-shape-policy: Banned function calls

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

`banned-function-call` is opt-in. Set a nonempty
`shapeOptions.bannedFunctionCall.functions` list to the SQL function names to
ban. An unqualified configured name matches the final name part, whether the
SQL call is qualified or unqualified: `pg_sleep` matches both `pg_sleep(...)`
and `pg_catalog.pg_sleep(...)`. A qualified configured name must match every
name part. Unquoted SQL and configured identifiers follow PostgreSQL folding
rules; quoted identifiers retain their exact case. Calls are matched from parsed function-expression facts, so
column references and aliases with the same spelling are not reported.

The check covers calls in CTEs, subqueries, mutations, native SQL, configured
executor calls, routine bodies, and statically recovered `EXECUTE` strings.
Plan-only `EXPLAIN` does not contribute calls; `EXPLAIN ANALYZE` does.
This is useful in parallel tests: `pg_sleep` occupies a database connection
for the duration of the delay and can stall other work waiting for a pooled
connection.

```yaml
rules:
  - rule: postgres-sql-shape-policy
    scope: repository
    include: ["**/*.test.ts", "test-helpers/**"]
    options:
      importSpecifier: "@example/db"
      trustedSqlTags: [{ module: "@example/db", name: sql }]
      bannedShapes: [banned-function-call]
      shapeOptions:
        bannedFunctionCall:
          functions: [pg_sleep, pg_catalog.pg_advisory_lock]
```

Each function entry may also be an object that limits the ban to clauses:

```yaml
shapeOptions:
  bannedFunctionCall:
    functions:
      - pg_sleep
      - name: uuidv7
        clauses: [where, join-on, having]
        hint: "compare the key against a stable bound computed once per statement"
```

A string entry bans every call and retains the existing diagnostic. An object
without `clauses` also bans every call. The optional `clauses` list accepts
`where`, `join-on`, `having`, `select-list`, `order-by`, `values`, `set`,
`default`, and `returning`. Calls inherit the innermost clause: a subquery's
WHERE overrides an outer SELECT list, and nested function arguments and CASE
expressions keep their containing clause. Column defaults in CREATE TABLE and
ALTER TABLE SET DEFAULT use `default`. ON CONFLICT assignments use `set`, its
arbiter and action WHERE predicates and partial-index WHERE predicates use `where`,
and MERGE WHEN conditions use
`where`. Recovered procedural IF, PERFORM, and RETURN expression wrappers and
window PARTITION BY or frame expressions stay unscoped. Genuine nested SELECT
clauses and window ORDER BY expressions retain their clauses.

Scoped findings name the clause, for example `uuidv7() is banned in WHERE`.
An optional `hint` appends `; <hint>` to the finding. Function names must be
nonempty and unique after PostgreSQL identifier decoding. An empty clause list,
unknown or duplicate clause, or blank hint is a configuration error. This policy
does not infer function volatility from the catalog.

For the scoped example above, `DEFAULT uuidv7()`, `VALUES (uuidv7())`,
`SELECT uuidv7()` and `UPDATE orders SET id = uuidv7()` pass. A predicate such as
`WHERE id >= uuidv7(INTERVAL '-30 days')` fails; compute a stable bound once per
statement and compare against that bound instead. Views, CTEs, routines and
recovered literal EXECUTE commands use the same clause facts.

An empty or omitted `functions` list is a configuration error when
`banned-function-call` is enabled. It has no default effect on the rule's
existing shapes. Unknown shape names remain configuration errors.

Counterexample: a test executor sleeps while holding a database connection.

```ts
await query(sql`SELECT pg_catalog.pg_sleep(30)`);
```

Fix: use a controlled test barrier or fixture synchronization instead of a
time delay. Suppress an intentional call with
`no-mistakes-disable-next-line postgres-sql-shape-policy` or
`no-mistakes-disable-line`; use
`no-mistakes-disable-file` only when the whole file is an intentional
exception.

```sql
SELECT pg_catalog.clock_timestamp();
```

This call remains allowed when only `pg_sleep` is configured.

[Back to the rule](postgres-sql-shape-policy.md)
