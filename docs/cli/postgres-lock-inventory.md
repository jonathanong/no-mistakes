# `no-mistakes postgres lock-inventory`

Report PostgreSQL lock sites in the configured scope and compare them with the
optional baseline configured on the `postgres-lock-inventory` rule.

```sh
no-mistakes postgres lock-inventory --root . --config .no-mistakes.yml
```

The command emits a structured report containing category counts, stable site
keys, baseline additions and stale entries, and unresolved SQL inputs. An
unresolved input is not lock-free and prevents baseline creation or pruning.
The exit status is 0 for a clean report or a successful baseline rewrite, and
1 when additions, stale entries, or unresolved inputs remain.

`--root` selects the project root. `--config` selects its no-mistakes
configuration. Set the baseline path in the rule's `options.baselineFile` (or
its `baseline` alias); it defaults to `.no-mistakes/postgres-locks.json`. There
is no command-line baseline path. A missing baseline is treated as empty, so
current sites appear as additions until the initial baseline is written.

## Write a baseline

Use `--write-baseline` only after reviewing the inventory:

```sh
no-mistakes postgres lock-inventory --root . --config .no-mistakes.yml --write-baseline
git add .no-mistakes/postgres-locks.json
no-mistakes check --root .
```

When the configured baseline does not exist, the explicit option creates the
initial baseline. A deleted tracked baseline must first be restored; deletion
cannot reset the ratchet. Add a new file to Git before checking it. For an existing
baseline, writing may prune stale removed-site entries, but refuses to accept
new sites or proceed while SQL remains unresolved. To accept an intentional
addition, copy its `file`, `category`, and `key` from the report's `added`
array into the baseline's `sites` array. Commit that entry alongside the
change that introduces the lock. Run the writer after reviewing any stale
entries you want to prune.

If unresolved executor SQL blocks adoption, use the rule's `include` and
`exclude` to start with paths whose SQL can be analyzed. Align `sqlInclude`
with the native SQL in that scope. A malformed baseline causes the standalone
command to exit with an error. The same input produces an `invalid-baseline`
finding during `no-mistakes check`.

Baseline keys use root-relative file, category, token-normalized statement,
and a duplicate ordinal within the file. Comments, whitespace, and line moves
do not change keys. SQL literal or operator changes and file renames produce
new keys. Adding one lock to a statement changes the keys of its other locks
because they share the statement fingerprint. Suppression directives do not
change inventory counts or baseline membership.

## Node API

The asynchronous Node API is:

```js
const report = await noMistakes.postgresLockInventory({
  root: ".",
  config: ".no-mistakes.yml",
  writeBaseline: false,
});
```

`writeBaseline` defaults to `false`. The API returns the same structured
inventory report as the CLI. `analyzeProject({ reports: [{ type: "check" }] })`
uses the same check behavior and suppression accounting as `check()`.
