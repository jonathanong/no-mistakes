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
their stable keys with an optional baseline. A site key includes its
root-relative file, category, token-normalized statement, and an ordinal for
duplicate identical statements in one file.

Comment-only, whitespace-only, and line-movement changes within a file do not
change a key. SQL token changes, including changes to literals or operators,
change the key. Renaming a file or changing the SQL creates a new site; the old
baseline key is then reported as stale. Duplicate ordinals distinguish identical
statements without using line numbers. Configured TypeScript helper
calls use the callee, argument count and static argument shapes, plus the
duplicate ordinal; changing argument values within the same shape does not
create a new helper site. SQL helper calls use the qualified function name.
Imported SQL sites are located at each configured executor call, so directives
at that call remain applicable. Distinct imported helper definitions in complete
alternatives retain separate sites, even when their function names match.

Counts cover the four row-lock strengths (`FOR UPDATE`, `FOR NO KEY UPDATE`,
`FOR SHARE`, and `FOR KEY SHARE`), the `NOWAIT` and `SKIP LOCKED` modifiers,
all recognized advisory-lock function prefixes, table locks, and sites found
through configured helpers. The report preserves the configured scope and
lists each unresolved SQL input separately. Stored function and procedure
bodies are outside the shared executable-SQL projection, so their source is
listed as unanalyzed and excluded from counts. Scope these files separately
and configure helper names for their call sites; baseline writing refuses any
unresolved source.

## Options

`baselineFile` selects the JSON baseline and defaults to
`.no-mistakes/postgres-locks.json`; `baseline` is its alias. Configure helper
TypeScript/SQL function names with `helpers`. TypeScript matches case exactly.
PostgreSQL folds unquoted names to lowercase. Quoted SQL names containing
uppercase letters require an exact configured spelling.
Executor selection uses the standard PostgreSQL options, including
`importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`,
`executorOptionProperties`, and `executorOptions`. Rule-level `include` and
`exclude` select source files; `sqlInclude` selects SQL files.

The baseline is compared with an empty inventory when it is missing, so every
current site is reported as added. With a baseline, newly added sites are
diagnostics. Stale-entry diagnostics are deferred while any SQL is unresolved,
because incomplete inventory cannot establish which baseline entries were
removed. Unresolved SQL is listed and prevents baseline creation or pruning.

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

Review whether the changed lock is intended and scoped correctly. If it is not,
change the SQL or remove the lock. An intentional new site remains an added-site
diagnostic: the baseline writer cannot accept additions. Resolve or configure
coverage for every unresolved SQL input before creating or pruning a baseline;
unresolved input cannot justify removing a baseline entry.

An initial baseline is created only with the explicit baseline-writing option.
Add the new file to Git, then run the check against that tracked baseline. A
missing baseline is treated as empty, so its current sites appear as additions
until initialized. An existing baseline can only shrink: writing refuses to
accept added sites or unresolved SQL. A deleted tracked baseline must be
restored before rewriting; deleting it cannot reset the ratchet. Review stale
entries before pruning them.
Do not use baseline writing to accept new sites.

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
