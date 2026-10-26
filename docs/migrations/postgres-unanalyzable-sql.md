# PostgreSQL `unanalyzableSql` for lock ordering, OFFSET, and generated writes

The shared `unanalyzable` policy is available at repository and rule
application scope. `unanalyzableSql` remains the SQL-specific alias; legacy
PostgreSQL parsers keep their `fail` spelling for the shared `report` policy.
When both forms are set at one scope, `unanalyzable` takes precedence.
PostgreSQL rules keep their existing defaults; this policy does not enable
low-effort analysis for them.

`postgres-lock-ordering`, `postgres-no-offset`, and
`postgres-no-generated-column-writes` now accept the `unanalyzableSql` option
used by the other PostgreSQL DML rules. Like those rules, it defaults to
`fail`. This is a behavior change: these three rules used to skip dynamic
executor SQL they could not check, and now report it with target
`unanalyzable` at the executor call.

Each rule reports only dynamic calls that could matter to it:

| Rule | Reported dynamic calls |
| ---- | ---------------------- |
| [`postgres-lock-ordering`](../rules/postgres-lock-ordering.md) | No SQL text was recovered. Recovered text with `FOR UPDATE` keeps the ordinary checks; recovered text without an exclusive lock clause is treated as non-locking. |
| [`postgres-no-offset`](../rules/postgres-no-offset.md) | No SQL text was recovered, or the recovered text contains a query (including query-bearing DML and CTEs), or is unknown or incomplete, without its own `OFFSET`. |
| [`postgres-no-generated-column-writes`](../rules/postgres-no-generated-column-writes.md) | The catalog applicable to the call has generated or trigger-maintained columns, and no SQL text was recovered, or the recovered text contains `INSERT`, `UPDATE`, or `MERGE` (including data-modifying CTEs), or is unknown or incomplete. Literal untracked targets do not exempt opaque SQL tails. |

To migrate, review each new `unanalyzable` finding:

- Make the executed SQL statically recoverable: pass a SQL literal, a
  `const` binding, or a configured trusted tagged template.
- For a one-off, add `no-mistakes-disable-next-line <rule>` above the executor
  call. For lock ordering, the configured safe directive (`deadlock-safe` by
  default) also works.
- To keep the earlier behavior, set the option explicitly:

```yaml
rules:
  - rule: postgres-no-offset
    scope: repository
    options:
      importSpecifier: "@example/db"
      unanalyzableSql: ignore
```

PostgreSQL 17 migration files can contain complete DDL forms that the bundled
SQL AST parser cannot represent. The schema checks recognize the exact grammar
of selected `ALTER TABLE`, `ALTER TYPE`, and `ALTER INDEX` forms from the
PostgreSQL regression suite and classify them as valid but unsupported. Under
the default policy these produce advice rather than an unanalyzable finding.
Recognition requires a complete statement, including its clause values and
closing delimiters; a malformed suffix remains a finding.
