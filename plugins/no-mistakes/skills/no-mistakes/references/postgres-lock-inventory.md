# PostgreSQL lock inventory

Enable `postgres-lock-inventory` only for explicitly selected executors, helper
names, and SQL files. `baselineFile` defaults to
`.no-mistakes/postgres-locks.json`; `helpers` defaults to an empty list.

Inspect the structured inventory before changing a baseline:

```sh
no-mistakes postgres lock-inventory --root . --config .no-mistakes.yml
```

The report includes category counts, stable site keys, added and removed sites,
and unanalyzed SQL. For SQL lock statements, comment-only, whitespace-only, and
line-movement changes within a file preserve a key; changing statement tokens,
including literals and operators, or renaming a file creates a new identity.
SQL helper keys use the qualified function name and duplicate ordinal, so
changes to helper argument tokens do not change those keys. Suppressions affect
ordinary check findings while the inventory retains physical sites.

After reviewing the initial inventory, `--write-baseline` can create a missing
baseline. Commit it. Restore a deleted tracked baseline before rewriting;
deletion cannot reset the ratchet. Subsequent writes may only prune removed
sites. To accept an intentional addition, copy its `file`, `category`, and
`key` from `added` into the baseline's `sites` array for review. Start with
rule `include`, `exclude`, and `sqlInclude` settings that select analyzable SQL
so unresolved inputs do not block initial adoption. Stored routine bodies
remain unresolved. Executable statements alongside them still count. Omit
per-rule `effort` and uncertainty options.

Use `await postgresLockInventory({ root, config, writeBaseline: false })` for
repeated in-process inventory queries. Ordinary `check()` and
`analyzeProject({ reports: [{ type: "check" }] })` expose the rule findings.

See the [rule contract](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/postgres-lock-inventory.md)
and [CLI options](https://github.com/jonathanong/no-mistakes/blob/main/docs/cli/postgres-lock-inventory.md).
