# PostgreSQL `unanalyzableSql` for lock ordering, OFFSET, and generated writes

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
| [`postgres-no-offset`](../rules/postgres-no-offset.md) | No SQL text was recovered, or the recovered text is a `SELECT` (or an unknown or incomplete statement) without its own `OFFSET`. |
| [`postgres-no-generated-column-writes`](../rules/postgres-no-generated-column-writes.md) | The catalog has generated or trigger-maintained columns, and no SQL text was recovered, or the recovered text is an `INSERT`, `UPDATE`, or `MERGE` (or an unknown or incomplete statement) that may target a tracked table. |

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
