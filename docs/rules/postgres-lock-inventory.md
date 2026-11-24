# `postgres-lock-inventory`

Opt-in inventory of configured PostgreSQL lock sites. It counts lock syntax at
configured executors and configured TypeScript/SQL helper functions, including
row-lock strengths, `NOWAIT`, `SKIP LOCKED`, advisory-lock calls, and `LOCK
TABLE`. It reports configured SQL that could not be analyzed as unresolved;
unresolved SQL is never counted as lock-free.

## Why and when

Use this rule to record the lock surface of selected code and review changes to
that surface over time. The baseline makes new lock sites visible for review;
it does not assert that an existing lock is safe or that a project has no locks.

## What it catches

The inventory reports physical lock sites in the configured scope and compares
their stable keys with an optional baseline. A SQL lock-statement key includes
its root-relative file, category, token-normalized statement, and an ordinal for
duplicate identical statements in one file. SQL helper keys use the qualified
function name and duplicate ordinal.

For SQL lock statements, comment-only, whitespace-only, and line-movement
changes within a file preserve keys. LF and CRLF line endings inside ordinary
and dollar-quoted multiline string literals are treated alike. The full statement supplies the locking context.
A token edit anywhere in that statement, including an added lock clause, creates
replacement keys for every lock site in it. A file rename also creates new
keys. The previous keys are reported as stale. Duplicate ordinals distinguish
identical statements without using line numbers.
Configured TypeScript helper calls use the callee, argument count and static
argument shapes, plus the duplicate ordinal; changing argument values within
the same shape does not create a new helper site. SQL helper calls use only the
qualified function name and duplicate ordinal, so changes to their argument
tokens do not change the helper key.
Imported SQL sites are located at each configured executor call, so directives
at that call remain applicable. Distinct imported helper definitions in complete
alternatives retain separate sites, even when their function names match.

Counts cover the four row-lock strengths (`FOR UPDATE`, `FOR NO KEY UPDATE`,
`FOR SHARE`, and `FOR KEY SHARE`), the `NOWAIT` and `SKIP LOCKED` modifiers,
all recognized advisory-lock function prefixes, table locks, and sites found
through configured helpers. The report preserves the configured scope and
lists each unresolved SQL input separately. A stored function or procedure body remains unresolved. Top-level executable
statements in the same source still contribute their sites. Configure helper
names for calls into stored routines.

## Options

`baselineFile` selects the JSON baseline and defaults to
`.no-mistakes/postgres-locks.json`; `baseline` is its alias. Automatic
configuration discovery reserves that default path for the baseline. Place a
custom baseline outside the `.no-mistakes/` fragment directory. Configure helper
TypeScript/SQL function names with `helpers`. TypeScript matches case exactly.
PostgreSQL folds unquoted names to lowercase. Quoted SQL names containing
uppercase letters require an exact configured spelling.
Executor selection uses the standard PostgreSQL options, including
`importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`,
`executorOptionProperties`, and `executorOptions`. Rule-level `include` and
`exclude` select source files; `sqlInclude` selects SQL files.
Configured `.query()` calls inside transaction callbacks are included.
Reassigning `.query` leaves an unresolved site instead of a counted lock.

The inventory uses its sole analysis mode at either [effort
setting](../configuration/README.md) and always reports incomplete SQL. Omit
`unanalyzable` and `unanalyzableSql` from its rule options.

The baseline is compared with an empty inventory when it is missing, so every
current site is reported as added. With a baseline, newly added sites are
diagnostics. Stale-entry diagnostics are deferred while any SQL is unresolved,
because incomplete inventory cannot establish which baseline entries were
removed. Unresolved SQL prevents baseline creation or pruning. Malformed baseline
contents produce an `invalid-baseline` finding at the baseline path during
`check`. Repair the file before running the standalone inventory.

## Valid example

This configuration scopes the inventory to application and migration SQL,
recognizes the configured executor, and follows the named helper:

```yaml
rules:
  - rule: postgres-lock-inventory
    scope: repository
    include: ["src/**/*.ts", "db/**/*.sql"]
    exclude: ["**/*.test.ts"]
    options:
      importSpecifier: "@example/db"
      executorNames: [query]
      executorFactoryNames: [beginTransaction]
      helpers: [reserveJob]
      sqlInclude: ["db/**/*.sql"]
      baselineFile: .no-mistakes/postgres-locks.json
```

```ts
import { query } from "@example/db";

void query("SELECT id FROM jobs WHERE id = $1", [1]);
```

The canonical option is `baselineFile`; `baseline` names the same setting.
Both the inventory and its counts include suppressed physical sites, so adding
a suppression cannot make a new site disappear from baseline comparison.

## Counterexample

Suppose the tracked baseline contains a statement locking `jobs`, and the
configured code adds a distinct lock on `accounts`:

```ts
import { query } from "@example/db";

void query("SELECT id FROM jobs WHERE id = $1 FOR UPDATE", [1]);
void query("SELECT id FROM accounts WHERE id = $1 FOR SHARE", [2]);
```

The `accounts` site is reported as added. A nearby line suppression does not
remove it from the inventory or make the baseline comparison pass.

## Fix

Review the lock and its scope. For an intentional change, run
`no-mistakes postgres lock-inventory --root .` and copy the reviewed site's
`file`, `category`, and `key` from `added` into the baseline's `sites`
array. Commit the baseline change with the SQL change. Review stale entries,
then run with `--write-baseline` to prune them. Restore a deleted tracked
baseline before rewriting it.

Use `--write-baseline` to initialize a missing baseline and add the file to Git.
For incremental adoption, start with application directories or specific TS and
SQL files whose executor SQL is statically analyzable. Narrow the rule's
`include` and `exclude` and keep `sqlInclude` aligned with the selected native
SQL files. Expand the scope as you resolve the remaining sources.

## Suppression

Standard file, line, and next-line `no-mistakes-disable` directives suppress
ordinary rule findings. Inventory counts and baseline membership always include
the physical sites, even when their findings are suppressed. Suppression cannot
hide an added site from the structured inventory or make a baseline update
acceptable.

## Related rules

Combine this inventory with
[`postgres-lock-written-rows-only`](postgres-lock-written-rows-only.md) to review
row-lock targets and transaction ownership, and
[`postgres-lock-ordering`](postgres-lock-ordering.md) to check acquisition order.
