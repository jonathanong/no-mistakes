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
| `table-lock` | `LOCK TABLE` in any mode without a proven later DDL purpose in the same transaction. | Use a bounded row lock, or make the DDL ordering explicit. |
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
Straight-line helpers can call other supported local helpers. Their parameterized
locks and writes are composed at each invocation through a known transaction,
including a caller that passes its `TransactionQuery` parameter onward. A type
annotation alone does not prove a transaction; unrelated transaction instances
and different keys remain distinct. Recursive helper cycles remain unresolved.

At the known transaction caller, helper effects require a direct
`await helper(...)` (including `return await`).
A plain `return helper(...)` or discarded promise is unanalyzable because an
`await using` transaction can be disposed before the helper finishes. Within a
summarized helper, each executor call must itself be awaited or returned. Helper
SQL must be a string literal or an interpolation-free untagged template. Tagged
templates and JavaScript interpolations remain unanalyzable in helper summaries.
Executor-option helpers described below also support configured SQL tags with
static parameter placeholders and a boolean parameter selecting between SQL arms.
SQL binding arrays must use identifiers or static primitive values. Compute
dynamic values at the caller and pass plain helper parameters; calls, assignments,
and property reads inside bindings could hide transaction side effects.
Executor shapes must match: `q(sql)` accepts `tx.query`, while `q.query(sql)`
accepts `tx`. A mismatch makes the entire helper call unanalyzable.
Use captured query functions directly in executor options; `.query` extraction
requires a transaction object alias.

Unsupported local and imported helpers receiving a configured transaction
executor retain the unanalyzable policy, including dynamic SQL, optional calls,
shadowed or mutated helpers, and bodies with other control flow. Named imported
helpers and re-exports use the same summaries through the prepared module resolver. Unresolved imports retain the unanalyzable policy.

`FOR UPDATE OF n, t, r, d` resolves every explicit alias to its base relation.
Each relation must have a matching update or delete in the same transaction;
writing only two of four locked relations leaves findings for the other two.
An unknown alias follows the unanalyzable policy.

An advisory key derived from the written row's id also passes the constant-key
check.

## Table locks that order DDL

A table lock can protect a preflight check before later DDL acquires its implicit
lock. The rule accepts a lock when the same transaction later runs `ALTER TABLE`
(including `ATTACH PARTITION` or `DETACH PARTITION`) or `DROP TABLE` on that
relation, or on a relation connected through partitioning, inheritance, or a
foreign key. Statically recovered `CREATE TABLE` statements in the transaction
can establish those relationships. Optional `schemaCatalogPath` supplies
existing partition, inheritance, and foreign-key relationships.

```ts
const tx = await beginTransaction();
await tx.query("LOCK TABLE events IN ACCESS EXCLUSIVE MODE");
await tx.query("SELECT 1 FROM events WHERE active LIMIT 1");
await tx.query("ALTER TABLE events DETACH PARTITION events_archive");
```

A guard relation need not be related to the DDL target. A read of the locked
relation after the lock and before later DDL also establishes the purpose:

```ts
const tx = await beginTransaction();
await tx.query("LOCK TABLE jobs IN SHARE MODE");
await tx.query("SELECT 1 FROM jobs WHERE pending LIMIT 1");
await tx.query("DROP TABLE events_archive");
```

These proofs require known transaction membership, unconditional execution,
and directly awaited executor calls in the same function. Nested executor calls
in a callee or argument do not establish a sequential DDL proof. Reads inside
CTE-bearing queries do not
establish a guard proof. A read before the lock or after the DDL does not protect
a preflight check. A lock
without later DDL, DML alone, or DDL in another transaction still reports. An
unrelated locked relation that is never read before the DDL also reports.

## Trigger-derived writes

Set optional `schemaCatalogPath` to a PostgreSQL schema catalog JSON file, relative
to the analyzed root, to include trigger effects in transaction write sets.
Trigger inference requires complete catalog coverage; ordering-only catalogs
provide DDL relationships without inferred trigger writes:

```yaml
options:
  importSpecifier: "@example/db"
  executorFactoryNames: [beginTransaction]
  schemaCatalogPath: schema.json
```

For example, the catalog can describe an `AFTER INSERT ON log` trigger whose
function performs `UPDATE r SET latest = NEW.id WHERE id = NEW.r_id`. This
transaction then writes the locked relation through that trigger:

```ts
const tx = await beginTransaction();
await tx.query('SELECT id FROM r WHERE id = $1 FOR UPDATE', [id]);
await tx.query('INSERT INTO log (id, r_id) VALUES ($2, $1)', [id, next]);
```

The triggering statement must write the trigger's source relation with the
matching `INSERT`, `UPDATE`, or `DELETE` event. An `UPDATE OF selected` trigger
contributes writes only when the statement assigns `selected`. An executor
assignment such as `SET total.amount = 1` matches `UPDATE OF total`, not
`UPDATE OF amount`. An executor
`INSERT ... ON CONFLICT DO UPDATE` also contributes its UPDATE event and assigned
columns; `DO NOTHING` does not. Trigger effects
can fire further triggers; cycles are guarded and traversal stops after 32
trigger hops.
Static writes in SQL and simple PL/pgSQL function bodies contribute their target
relations. These facts establish relation writes without claiming a proven
matching row key. A trigger on an unwritten source table, an unmatched event or
column, a `WHEN` condition, or a body that cannot be analyzed contributes no
write evidence, so an otherwise unwritten lock remains reported. Without a
catalog, the rule uses only its existing direct and helper write evidence.
The catalog generator omits disabled and replica-only triggers; unattached
functions and dropped triggers supply no write evidence. Transactions that change
`session_replication_role` use only direct write evidence: the catalog does not
distinguish origin-only triggers from always-enabled triggers. A `set_config`
call whose setting name is not a plain string literal also disables inferred
trigger writes for that transaction, since it could change
`session_replication_role`; direct row write proofs remain available.

Known limits: transaction-local `ALTER TABLE ... DISABLE TRIGGER` and
`DROP TRIGGER` statements are not tracked against the catalog snapshot. Row-level
trigger effects count as relation-write evidence even when the triggering
statement matches zero rows; the rule does not prove affected-row counts.

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
`using`, or `await using`. A direct, non-optional call of an imported configured
factory also recognizes an arrow or function expression in its first argument:
the callback's first simple identifier parameter is the transaction. Member
callees, shadowed factory names, referenced callbacks, and destructured callback
parameters do not establish this proof. Reassigning the callback parameter
invalidates its transaction proof. Case-block shadows apply only after the
switch discriminant, which is evaluated in the outer environment. With an import specifier, executor names
default to `query`, `read`, and `write`.
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

`schemaCatalogPath` defaults to absent. When supplied, it names a root-relative
PostgreSQL schema catalog used to connect DDL targets to locked relations
through partitioning, inheritance, and foreign keys.
Guard-read proofs do not require a catalog. Ordering-only catalogs are accepted,
but absent relationship metadata cannot prove that two relations are linked.
Only complete catalogs contribute trigger-derived write evidence.

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

Runs at `max` only. Configuring `effort: low` is accepted: `check` warns `requested effort low; effective effort max because this rule has no low implementation` and still runs at `max`. The rule checks configured TS/JS executor calls and
transaction-local write sets. Standalone SQL files are outside this rule’s scope.
SQL or transaction membership that cannot be resolved follows the shared
unanalyzable policy: `unanalyzable: report` (default) fails, `advise` adds a
non-failing advisory, and `ignore` omits the diagnostic. The legacy
`unanalyzableSql` option accepts `fail`, `advise`, and `ignore`; `fail` means
`report`. These policies never certify an unknown transaction as safe.
Dynamic relation names, key columns, aliases, and `FOR UPDATE OF` targets also
follow this policy; recovered SQL placeholders cannot establish their identity.
A possible earlier return or throw prevents a later write from proving that a
locked row is written while the transaction remains open. A branch that awaits a commit or rollback
and returns ends that transaction path; a matching write on the remaining
fall-through path can still satisfy the lock. An uncommitted return, a boundary that is not awaited, a commit on a continuing
path, or an explicit SQL transaction boundary remains unresolved.
Early exits require a non-optional awaited close proved in the same function for the original
transaction binding. Every path through the returning arm must close it, including
both arms of a nested `if`.
Closes inside loops, logical or conditional expressions, switch cases, or
`try`/`catch`/`finally` remain unresolved. Rewrite them as explicit awaited closes
in ordinary blocks or fully covered `if` branches. Reassignment and overwritten
boundary methods do not establish closure.
Recovery is conservative when another live transaction remains open on the
returning path: later writes retain the unresolved policy. Keep matching writes
on every continuing transaction path, or remove the lock.

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
Unchanged local `const` aliases of a factory result retain the same transaction
identity for direct calls and `.query` calls, independently of `executorOptions`
and `executorOptionProperties`:

```ts
await using transaction = await beginTransaction();
const query = transaction;
await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
await transaction('UPDATE items SET active = true WHERE id = $1', [id]);
```

A reassigned `let` alias or an alias passed to another function before execution
does not prove transaction membership. Keep the executor in an unchanged local
`const` binding to make the lock and matching write provable.

Pool aliases, conditional transaction/pool receivers, property or destructured
receivers, and aliases read before transaction initialization do not establish
transaction membership. A lock reached through one of these forms retains an `unanalyzable` finding
under `unanalyzableSql: fail`; the configured unanalyzable policy still applies. An alias also cannot extend a transaction past commit or resource
disposal; a later matching write does not make that lock safe. Saved negative
controls cover these boundaries with executor-option selectors off and on.
Unrelated callbacks remain outside this scope when stored beside configured
executors, including numeric/computed/default destructuring, empty holders, and
mutable holders.

`executorOptions` defaults to `[]`. Configure the properties that select a client
at your imported executors. No option-property routing is inferred without this
configuration. These names select properties in an options argument; bare
executor binding resolution remains independent, including same-spelled aliases:

```yaml
executorOptions:
  - name: query
```

A bare call remains a binding lookup even when its name matches a configured
property. This call has no options argument to select:

```ts
await using transaction = await beginTransaction();
const query = transaction;
await query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
```

Names must be nonempty. The existing `executorOptionProperties: [query]` form
remains supported. Both lists combine, and repeated names are deduplicated.
An executor can accept an inline options object that routes SQL to a transaction:

```ts
await using tx = await beginTransaction();
await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], { query: tx });
await query('UPDATE items SET active = true WHERE id = $1', [id], { query: tx });
```

A configured property may hold the transaction binding or its `.query` member.
The options object can also be a local `const` object literal passed later,
including a shorthand property referring to an immutable transaction-query alias:

```ts
const query = tx.query;
const options = { query };
await write('UPDATE items SET active = true WHERE id = $1', [id], options);
```

Unknown values, getters, computed properties, spreads, mutations, and conflicting executor
overrides follow the unanalyzable policy; they never establish autocommit
membership. Use a static object and the configured transaction binding to make
routing clear.

Local, unexported straight-line helpers can forward an optional options parameter
to a configured executor. With `sql` configured in `trustedSqlTags`, for example:

```ts
async function lockItem(id: string, opts?: { query?: Client }) {
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, opts);
}
async function readItem(id: string, forUpdate: boolean, opts?: { query?: Client }) {
  await read(forUpdate
    ? sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`
    : sql`SELECT id FROM items WHERE id = ${id}`, opts);
}

const tx = await beginTransaction();
await lockItem(id, { query: tx });
await readItem(id, true, { query: tx });
await readItem(id, false);
await write('UPDATE items SET active = true WHERE id = $1', [id], { query: tx });
```

The rule checks each resolved caller. Literal boolean arguments select the SQL
arm, so a non-transaction caller passing `false` does not enable the lock. An
unknown flag cannot prove that the lock is disabled. Every caller that enables
the lock must supply a static inline options object selecting a proven
transaction; missing options, a non-transaction
client, and unresolved callers retain findings. At least one resolved caller is
required; a helper with no resolved callers cannot prove transaction membership.
A proven transaction caller never hides an unresolved helper invocation,
regardless of the order of those calls.
A transaction binding reassigned to a pool before the helper call also fails
that proof. A `Client` type annotation alone
does not prove a transaction.
Passing a `const` options binding to these helpers remains unresolved; pass the
static object inline instead. Direct executor calls retain the `const` options
support described above.
Exported or escaped option helpers retain findings because external callers
cannot be proven here. An opaque whole-options parameter forwarded to an imported
executor keeps that executor's default non-transaction classification until a
supported caller proves routing. Its row lock reports
`row-lock-outside-transaction`; an explicit unknown property such as
`{ query: unknownClient }` still follows the unanalyzable policy. Pass inline
options at a known transaction call site instead. This extension does not generalize arbitrary branches or dynamic SQL.

### Stale SQL suppressions

Aggregate `check` audits `no-mistakes-disable-next-line` comments inside
statically recovered executor SQL, as well as native SQL and host comments.
When the statement becomes accepted, remove its stale directive; otherwise
`unused-disable-directive` identifies the physical comment line. A directive
that still suppresses a finding remains used. Unexecuted SQL examples are not
executor SQL. See [suppression hygiene](../configuration/suppressions.md).

## Invalid UTF-8

Source text this rule reads, including standalone SQL, is decoded lossily, so a stray byte does not drop the file or replace it with an encoding failure. Other read errors contribute no source text. `unanalyzableSql` still applies to SQL the statement pass cannot parse.
