# `postgres-lock-written-rows-only`

<!-- cspell:ignore xact hashtext hashtextextended -->

Opt-in PostgreSQL transaction lock checks. Findings point at the lock statement;
static helper locks are reported at their caller, with the helper lock line.
There is no autofix because changing a transaction's locking needs application
review.

## Why and when

Locking parents, ancestors, or a shared global key widens contention and the
possible deadlock surface without protecting rows the transaction changes.
Prefer locks on the rows being updated or deleted. An insert's foreign-key
check already acquires the required parent KEY SHARE lock.

## What it catches

| Check id | Behavior | Rewrite |
| --- | --- | --- |
| `row-lock-unwritten-relation` | `FOR UPDATE`, `FOR NO KEY UPDATE`, `FOR SHARE`, or `FOR KEY SHARE` on a relation the same transaction never updates or deletes; provable keys must match too. | Lock the written row, or rely on the foreign-key check. |
| `table-lock` | `LOCK TABLE` in any mode. | Use a bounded row lock on the written row. |
| `constant-advisory-lock` | Literal or constant keys (including builtin `hashtext` / `hashtextextended` of constants) to `pg_advisory_lock`, `pg_advisory_xact_lock`, their `try_` forms, and two-key variants. | Use a key derived from the written entity. |
| `row-lock-outside-transaction` | A row lock on a known non-transaction executor, unless the same SQL statement writes the locked relation. | Move the lock and write into one transaction. |

Each check defaults to `advise`. `fail` emits a failing finding and `ignore`
disables that check. Never interpret an unresolved statement as a proven write.

For the examples below, opt in with explicit failing severities:

```yaml
rules:
  - rule: postgres-lock-written-rows-only
    scope: repository
    options:
      importSpecifier: "@example/db"
      executorFactoryNames: [beginTransaction]
      checks:
        row-lock-unwritten-relation: fail
        table-lock: fail
        constant-advisory-lock: fail
        row-lock-outside-transaction: fail
```

## Valid example

With `beginTransaction` configured in `executorFactoryNames`:

```ts
import { beginTransaction } from '@example/db';

export async function updateChild(childId: string, name: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM children WHERE id = $1 FOR UPDATE', [childId]);
  await tx.query('UPDATE children SET name = $2 WHERE id = $1', [childId, name]);
}
```

The same relation and key are locked and updated within one transaction.
A top-level helper with a straight-line body of static executor calls can accept
an executor parameter. Each invocation through a known transaction substitutes
its immutable argument bindings before comparing the helper's locked relations
and keys with the caller's writes. For example, `lockUser(tx.query, userId)`
requires an update or delete of that same user in `tx`; an update of a membership
does not satisfy the lock. Diagnostics name the helper and its lock line.
Helper effects require a direct `await helper(...)` (including `return await`).
A plain `return helper(...)` or discarded promise is unanalyzable because an
`await using` transaction can be disposed before the helper finishes. Within a
summarized helper, each executor call must itself be awaited or returned.
Executor shapes must match: `q(sql)` accepts `tx.query`, while `q.query(sql)`
accepts `tx`. A mismatch makes the entire helper call unanalyzable.

Unsupported local and imported helpers receiving a configured transaction
executor retain the unanalyzable policy, including dynamic SQL, optional calls,
shadowed or mutated helpers, and bodies with other control flow. Named imported helpers and re-exports use the same summaries through the prepared
module resolver. Unresolved imports retain the unanalyzable policy.

An advisory key derived from the written row's id also passes the constant-key
check.

## Counterexample

```ts
import { beginTransaction } from '@example/db';

export async function insertChild(parentId: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM parents WHERE id = $1 FOR UPDATE', [parentId]);
  await tx.query('INSERT INTO children (parent_id) VALUES ($1)', [parentId]);
}
```

The transaction never updates or deletes `parents`. Inserting a child does not
make its parent part of the write set.

Other counterexamples, at configured executor calls:

```ts
import { beginTransaction, query } from '@example/db';

export async function broadLocks(childId: string) {
  const tx = await beginTransaction();
  await tx.query('LOCK TABLE children IN SHARE MODE');
  await tx.query('SELECT pg_advisory_xact_lock(42)');
  await tx.query('SELECT pg_try_advisory_lock(1, 2)');
  await query('SELECT id FROM children WHERE id = $1 FOR UPDATE', [childId]);
}
```

The final call uses a known non-transaction executor, so its row lock is released
at the end of that statement.

## Hashed keys and atomic statements

At configured executors, `pg_advisory_xact_lock(hashtext('global'))` and
`pg_advisory_xact_lock(hashtextextended('global', 0))` serialize unrelated rows.
The rule recognizes both unqualified and `pg_catalog`-qualified hash functions,
including module `const` values passed as SQL parameters. Every argument,
including the extended hash seed, must be proven constant. A row-derived bound
parameter, such as `hashtextextended($1, 0)` with `[rowId]`, remains compliant.
Arbitrary functions are outside this constant proof.

An autocommit queue claim can safely lock and write in one statement:

```sql
WITH candidates AS (
  SELECT id FROM jobs ORDER BY id LIMIT 50 FOR UPDATE SKIP LOCKED
), claimed AS (
  UPDATE jobs SET claimed_at = now() FROM candidates
  WHERE jobs.id = candidates.id RETURNING jobs.id
)
SELECT id FROM claimed
```

This passes because the same statement writes `jobs`. A write to another
relation, a separate SQL statement in the same string, or a later executor call
does not exempt the lock. CTE names cannot establish a base relation identity.
The ordinary `SELECT ... FOR UPDATE` autocommit finding remains enabled.

## Fix

Remove the parent lock and rely on the foreign-key check. When a read precedes
an update, lock the exact row to update, using the same column and bound value.
Replace table/global locks with entity-specific coordination and keep row locks
inside the transaction that performs the write.

## Options and defaults

```yaml
rules:
  - rule: postgres-lock-written-rows-only
    scope: repository
    options:
      include: ["src/**/*.ts"]
      importSpecifier: "@example/db"
      executorNames: [query]
      executorFactoryNames: [beginTransaction]
      checks:
        row-lock-unwritten-relation: fail
        table-lock: fail
        constant-advisory-lock: advise
        row-lock-outside-transaction: fail
      unanalyzableSql: advise
      allow:
        - relation: maintenance_jobs
          file: src/maintenance.ts
          line: 18
          reason: Serializes a documented maintenance protocol
      writeSets:
        - file: src/jobs.ts
          transactionLine: 12
          relations: [jobs]
          reason: The transaction calls a write helper
```

The rule inherits the selected project's `postgres` executor scope. Without
project scope, configure `importSpecifier` or explicit `executorNames`.
`include` and `exclude` default to empty and narrow the selected file scope.
`executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags` default to
empty; factories identify transaction executor bindings declared with `const`,
`using`, or `await using`. Callback transaction membership is unresolved and
follows the unanalyzable policy. With an import specifier, executor names default to `query`, `read`, and `write`.
`executorNames` replaces that list; use `extraExecutorNames` to extend it instead.
The two options are mutually exclusive, including when either list is empty.
`extraExecutorNames` defaults to absent and `excludeExecutorNames` defaults to
`[]`; exclusions remove names after replacement or defaults plus extras are
selected. For example, `extraExecutorNames: [readPool]` with
`excludeExecutorNames: [query]` selects `read`, `write`, and `readPool`.
Factory/type receiver selection remains independent of these imported-executor
names. See [executor selection](../configuration/rules.md#postgresql-executor-selection)
for explicit empty lists and configuration warnings. Declare these options in
the project's PostgreSQL scope when the rule inherits one.

`checks` defaults to all four ids at `advise`. `allow` defaults to `[]`;
selectors are optional `relation`, `key`, `file`, and `line`, with a required
`reason`. At least one selector is required and all supplied selectors must
match. `file` is an exact root-relative path with `/` separators, not a glob.
`relation` uses the canonical PostgreSQL name (unquoted names fold to lowercase;
quoted case is preserved). `key` matches the row predicate’s SQL value (for example `$1`) or the canonical
SQL constant display for an advisory lock. Two advisory keys are joined by
`, `. Use narrow selectors.

`writeSets` defaults to `[]`; each entry names
a `file`, the transaction factory's `transactionLine`, and extra written
`relations`, with an optional `reason`. It is an explicit assertion for helper
writes, not evidence of a matching key. Unused allow and write-set entries are
reported so stale exceptions do not silently accumulate. A write-set entry is
used only when it covers an otherwise-unwritten row lock.

### Analysis and unresolved SQL

`effort: max` is the default; `low` uses syntax proofs without the cross-file
max-effort evaluator. The rule checks configured TS/JS executor calls and transaction-local write
sets. Standalone SQL files are outside this rule’s scope. SQL or transaction membership that cannot be resolved follows the shared
unanalyzable policy: `unanalyzable: report` (default) fails, `advise` adds a
non-failing advisory, and `ignore` omits the diagnostic. The legacy
`unanalyzableSql` option accepts `fail`, `advise`, and `ignore`; `fail` means
`report`. These policies never certify an unknown transaction as safe.
Dynamic relation names, key columns, aliases, and `FOR UPDATE OF` targets also
follow this policy; recovered SQL placeholders cannot establish their identity.
A possible earlier return or throw also prevents a later write from proving
that a locked row is written. Keep the matching write on every path that takes
the lock, or remove the lock.

The rule does not establish advisory-key provenance from unwritten relations,
unbounded recursive read sets, duplication of unique-index guarantees, or
lock ordering across transactions. Key equality is checked only when provable;
unresolved inputs follow the selected policy rather than guessed equivalence.

## Suppression

Use `no-mistakes-disable-next-line postgres-lock-written-rows-only` immediately
before the executor call, `no-mistakes-disable-line` on that line, or
`no-mistakes-disable-file` for a justified file exception. Add a reason:

```ts
// no-mistakes-disable-next-line postgres-lock-written-rows-only: reviewed maintenance protocol
await tx.query('LOCK TABLE maintenance_jobs IN EXCLUSIVE MODE');
```

## Related rules

[`postgres-bounded-statements`](postgres-bounded-statements.md) checks statement
bounds. [`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) restricts SQL
shapes. See [executor bindings](../postgres-facts/executor-bindings.md) for
configured executor ownership.

### Resource-scoped transactions and executor options

Configured factory results bound with `using` or `await using` retain transaction
membership until the enclosing block ends, just like an immutable `const` binding.
Nested blocks and shadowed bindings keep distinct transaction identities.

`executorOptionProperties` defaults to `[]`. Configure explicit property names
(such as `[query]`) when an executor accepts an inline options object that routes
SQL to another executor:

```ts
await using tx = await beginTransaction();
await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], { query: tx });
await query('UPDATE items SET active = true WHERE id = $1', [id], { query: tx });
```

A configured property may hold the transaction binding or its `.query` member.
Unknown values, computed properties, spreads, and conflicting executor overrides
follow the unanalyzable policy; they never establish autocommit membership. Use an
inline static object and the configured transaction binding to make routing clear.
