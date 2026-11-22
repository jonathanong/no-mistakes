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
no-mistakes postgres lock-inventory --root . --config .no-mistakes.yml
```

When the configured baseline does not exist, the explicit option creates the
initial baseline. A deleted tracked baseline must first be restored; deletion
cannot reset the ratchet. Add a new file to Git before checking it. For an existing
baseline, writing may prune stale removed-site entries, but refuses to accept
new sites or proceed while SQL remains unresolved. New sites remain diagnostics;
the writer cannot accept them. Review every addition and stale entry before
changing code or pruning the baseline.

Baseline keys use root-relative file, category, token-normalized statement,
and a duplicate ordinal within the file. Comments, whitespace, and line moves
do not change keys. SQL literal or operator changes and file renames produce
new keys. Suppression directives do not change inventory counts or baseline
membership.

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
