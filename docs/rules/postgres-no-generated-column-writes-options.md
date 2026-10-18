# Generated Column Write Options

## Options and defaults

The rule compiles its options into two internal inputs, not nested YAML
objects: the schema catalog and embedded-SQL matcher. There are no direct
`schema` or `embedded` options.

- `sqlInclude` supplies the schema catalog's SQL-file globs. An omitted or
  empty list defaults to `**/*.sql`.
- `include` selects files containing DML. When omitted or empty, it analyzes
  `.ts`, `.mts`, `.tsx`, `.js`, and `.sql` files; otherwise its glob list is
  used.
- `importSpecifier` supplies the embedded-SQL matcher's import source. When
  omitted or empty, it has no module default.
- `executorNames` supplies the imported executor names that contain SQL. An
  omitted or empty list uses `[query, read, write]` only with a configured module.
- `extraGeneratedColumns` adds `{ table, column }` pairs to the generated
  column catalog. It defaults to an empty list.
- `triggerMaintainedColumns` defaults to `[]`. A non-generated column whose
  name matches an entry, on any table in schema SQL, is treated as set by the
  database. An empty name or a case-insensitive duplicate is a config error.
  An entry that matches no column is a stale finding. `[]` leaves today's
  generated-column findings unchanged.
- `unanalyzableSql` defaults to `fail` (`fail`, `ignore`, or `advise`; other values are a
  configuration error). `ignore` skips dynamic executor SQL that cannot be
  checked instead of reporting it.

### Unanalyzable SQL

**Behavior change:** this rule previously dropped dynamic executor SQL unless
its recovered text already proved a write. With the default
`unanalyzableSql: fail`, a dynamic TypeScript executor call without complete alternatives is reported with
target `unanalyzable` at the executor call when it could assign a generated or
trigger-maintained column:

- no SQL text was recovered (an opaque `write(sql)` argument or a
  `write(cond ? a : b)` choice whose arms cannot all be recovered), or
- the recovered text contains `INSERT`, `UPDATE`, or `MERGE`, including a
  data-modifying CTE, or its leading statement is unknown or incomplete.

Recovered keyword checks ignore quoted text, nested PostgreSQL block comments,
and keyword text within identifiers, including `$` and Unicode continuations.

Dynamic `SELECT` and `DELETE` text without a recovered column write is not
reported. Literal untracked targets do not exempt an opaque write: its tail
could introduce another statement targeting a protected column. Nothing is reported
when the schema catalog has no generated or trigger-maintained columns. Columns
that recovered text already proves are still reported with their ordinary
findings. See the [migration note](../migrations/postgres-unanalyzable-sql.md).

```ts
import { write } from "@example/db";

// Counterexample: both calls are reported as unanalyzable by default.
export function run(sql: string) {
  return write(sql);
}

export function touch(column: string) {
  return write(`UPDATE items SET ${column} = now()`);
}
```

Fix: pass a SQL literal or a configured trusted tagged template so the written
columns can be checked, suppress the executor line with
`no-mistakes-disable-next-line postgres-no-generated-column-writes`, or set
`unanalyzableSql: ignore` to restore the earlier skip.

### Executor configuration

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `importSpecifier` to your database module or list `executorNames` explicitly.
Use `executorNames: []` without a module to select no executor calls and retain
SQL-file/native-SQL analysis where supported. See the
[executor migration](../migrations/explicit-postgres-executors.md).

| Option            | Default                                                      | Behavior                                                                            |
| ----------------- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `importSpecifier` | Empty                                                        | Set explicitly to your database module to match its named imports.                  |
| `executorNames`   | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |
| `executorFactoryNames` | Empty | Named imports (from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty) whose call result bound with `const`, `let`, `using`, or `await using` is an executor inside the declaring block. |
| `executorTypeNames` | Empty | Imported type names (`import type` or inline `type` specifiers) from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty, whose annotated parameters, including optional and inline-destructured ones, are executors inside the declaring function. |
| `trustedSqlTags` | Empty | Named imports of `name` from `module`, or a subpath of `module`, are parameterized SQL tags. A renamed local binding is trusted. A default import is not. A shadowed or rebound local fails closed. The same name from another module, or a sibling prefix such as `@example/dbx`, stays untrusted. |
| `reportUnmatchedExecutorNames` | `false` | Opt in to one finding per `executorFactoryNames` / `executorTypeNames` entry that none of the files this rule scans imports from `importSpecifier`, its subpaths, or a relative path that resolves into that package (a typo or a wrong module silently disables checking). Reported against the config file, so line suppression does not apply; remove the entry or leave this off to silence it. Rust rule only. See [Reporting unmatched names](../migrations/explicit-postgres-executors.md#reporting-unmatched-names). |

`executorFactoryNames` and `executorTypeNames` add scoped executors: `tx` in
`await using tx = await openTransaction()` or `run` in `run: TxExecutor` is scanned
only inside the block or function that declares it, and a same-named identifier
elsewhere is not. Matching is by configuration only, both default to empty, and
neither counts as executor selection: still set `importSpecifier` or `executorNames`.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`. A relative import (`../transaction`) matches only when the resolved
file is inside the package `importSpecifier` resolves to; a same-named import outside
that package does not. If the package root cannot be determined, relative imports do
not match. `executorNames` still requires the exact module.

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

