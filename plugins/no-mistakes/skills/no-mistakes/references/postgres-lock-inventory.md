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
sites; they reject new sites or any unanalyzed SQL. Stored routine definitions
remain unanalyzed because their bodies are outside the shared executable-SQL
projection. Scope those sources separately and configure helper call names.

Use `await postgresLockInventory({ root, config, writeBaseline: false })` for
repeated in-process inventory queries. Ordinary `check()` and
`analyzeProject({ reports: [{ type: "check" }] })` expose the rule findings.

See the [rule contract](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/postgres-lock-inventory.md)
and [CLI options](https://github.com/jonathanong/no-mistakes/blob/main/docs/cli/postgres-lock-inventory.md).
