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
| `row-lock-unwritten-relation` | `FOR UPDATE`, `FOR NO KEY UPDATE`, `FOR SHARE`, or `FOR KEY SHARE` on a relation or provable key the same transaction never writes. | Lock the written row, or rely on the foreign-key check. |
| `table-lock` | `LOCK TABLE` in any mode without a proven later DDL purpose in the same transaction. | Use a bounded row lock, or make the DDL ordering explicit. |
| `constant-advisory-lock` | Literal or constant keys (including builtin `hashtext` / `hashtextextended` of constants) to `pg_advisory_lock`, `pg_advisory_xact_lock`, their `try_` forms, and two-key variants. | Use a key derived from the written entity. |
| `row-lock-outside-transaction` | A row lock on a known non-transaction executor, unless the same SQL statement writes the locked relation. | Move the lock and write into one transaction. |
| `share-lock-then-write` | A known `FOR SHARE` followed by an update or delete, or `FOR KEY SHARE` followed by a delete or catalog-proven key-column update, on the same row in the same transaction. | Choose a lock mode that matches the write, or use the conditional write directly. |
| `redundant-prelock` | Advisory-only: a discarded `SELECT 1 FOR UPDATE` immediately precedes a conditional write with the same provable key. | Remove the unused pre-lock and let the conditional write decide the race. |

The four established checks default to `advise`. The two sequence checks are
opt-in and stay disabled unless listed in `checks`. `fail` emits a failing
finding for `share-lock-then-write`; `redundant-prelock` accepts only `advise`
or `ignore`, and configuration with `fail` is invalid. `ignore` disables a
check. Never interpret an unresolved statement as a proven write.

## Ban advisory locks

`constant-advisory-lock` reports only keys proven constant. To ban advisory
locks regardless of key, use `postgres-sql-shape-policy` with its
`banned-function-call` shape and list all eight PostgreSQL function names:

```yaml
rules:
  - rule: postgres-sql-shape-policy
    scope: repository
    options:
      bannedShapes: [banned-function-call]
      shapeOptions:
        bannedFunctionCall:
          functions:
            - pg_advisory_lock
            - pg_advisory_lock_shared
            - pg_advisory_xact_lock
            - pg_advisory_xact_lock_shared
            - pg_try_advisory_lock
            - pg_try_advisory_lock_shared
            - pg_try_advisory_xact_lock
            - pg_try_advisory_xact_lock_shared
      exclude: [src/intentional-locks.ts]
```

This shape matches parsed function calls, including schema-qualified names.
Scope paths with its `include` and `exclude` options; there is no per-function
allow-path option for this shape.

## Locking CTE that writes another relation

A lock inside a CTE remains associated with its base relation. Writing another
table in the statement does not count as updating or deleting the locked rows:

```sql
WITH parent AS MATERIALIZED (
  SELECT o.id FROM orders AS o WHERE o.id = $1 FOR UPDATE OF o
)
INSERT INTO order_events (order_id) SELECT id FROM parent;
```

If the same known transaction does not write the locked row in `orders`,
`row-lock-unwritten-relation` reports the lock. A later write to `orders` in
that transaction satisfies the check only when it writes the locked row with a
matching provable key. CTE names do not establish a base relation identity.

## Share lock followed by a write

Enable this check with `checks.share-lock-then-write: advise` (or `fail`). It
reports a `FOR SHARE` when a later statement in the same known transaction
updates or deletes the same row. A relation-wide lock or write can also match.
Comparable equality predicates must have the same bound key. When the check
cannot compare the keys, it follows `unanalyzableSql`. An earlier proven
`FOR UPDATE` on the same row already holds the stronger lock.

`FOR KEY SHARE` matches a later delete or an update that can change a key
column of the locked relation. Set `schemaCatalogPath` to identify key columns
from valid, immediate, non-partial unique indexes on plain columns. The check
uses explicit assignment targets, including tuple assignments. An assignment
such as `SET id = id` preserves the key, and non-key updates are compatible
with `FOR KEY SHARE`. The catalog is needed for key-column updates.

```sql
SELECT 1 FROM accounts WHERE id = $1 FOR SHARE;
UPDATE accounts SET balance = balance - $2 WHERE id = $1;
```

Choose the lock strength required by the later write before reading, or remove
the pre-lock when a conditional write already decides whether the row changes.
The check relies on direct awaited SQL calls with known transaction membership
and statement order. A write to the locked relation whose promise is awaited
through a variable or discarded follows `unanalyzableSql` because the check
cannot establish its order relative to the lock. Locks and
writes recovered from helpers, dynamic SQL, or unresolved control flow also
follow `unanalyzableSql` when execution order cannot be proved.
`FOR UPDATE` and `FOR NO KEY UPDATE` are not shared-lock candidates, and a write
directly awaited before the lock does not match this sequence.

## Redundant pre-lock before a conditional write

This check is advisory-only. Set `checks.redundant-prelock: advise` to report a
direct, awaited `SELECT 1` or a plain projection of the equality key with
`FOR UPDATE` whose result is discarded when the next direct awaited SQL call is
a conditional `UPDATE` or supported upsert with
the same statically proven unique row key. An `UPDATE` requires a matching
unique key in the complete catalog supplied through `schemaCatalogPath`.
For a supported upsert with one `VALUES` row, a single-column `ON CONFLICT`
target establishes the unique key. A fully decoded multi-row upsert that
targets the locked relation and the same stable key in the transaction
can produce an `unanalyzable` diagnostic under `unanalyzableSql`. The
redundancy advisory requires a single-row upsert. It accepts `FROM ONLY` or a
plain `FROM` for an upsert when a complete catalog proves the table has no
descendants. The conditional `UPDATE` advisory requires `FROM ONLY`.
The rule abstains when it cannot prove that the selected values are
unused, the key predicates match, the key binds stay unchanged, and both calls
belong to the same known transaction. An awaited write can be the first
initializer in a declaration with multiple bindings. An earlier initializer
may observe data or affect execution, so the proof stops there. Intervening
reads, consumed results, computed projections, unconditional writes, and
unresolved transaction membership do not produce this finding. `fail` is an
invalid severity for this check. An unresolved proof may still produce an
unanalyzable diagnostic according to `unanalyzableSql`.

```sql
SELECT 1 FROM ONLY jobs WHERE id = $1 FOR UPDATE;
UPDATE jobs SET state = 'done' WHERE id = $1 AND state = 'running';
```

The conditional write can match zero rows. In that case, the pre-lock may
still protect work later in the transaction. The check skips the suggestion
when a later statement in the same transaction can access the locked relation.

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
An unconditional `INSERT INTO table (columns...) VALUES (...) ON CONFLICT (column) DO UPDATE SET ...`
in that transaction can also satisfy a row lock when all of the following hold:

- The lock targets the same relation, column, and key value. The select list
  does not affect write-set membership. `FROM ONLY` establishes the physical
  scope directly. A lock without `ONLY` requires a complete catalog with
  `hasDescendants: false` on that relation. Wider or unknown physical scope
  produces an `unanalyzable` diagnostic.
- The `ON CONFLICT` clause targets a single plain column (no expressions,
  operator classes, or `COLLATE` clauses) with no target `WHERE` predicate, and
  `DO UPDATE` has no `WHERE` clause.
- The `INSERT` has no `OVERRIDING USER VALUE`, and its `VALUES` query has no
  `WITH`, `LIMIT`, or `FETCH` clause.
- Every expression in every `VALUES` row is a plain literal (number,
  single-quoted string, or boolean) or a bind placeholder (`$n`). Non-key
  columns can also use `DEFAULT`. Every `DO UPDATE SET` right-hand side is an
  identifier, qualified identifier such as `EXCLUDED.state`, or plain
  literal/placeholder value.

This applies to both single-row and multi-row `VALUES`, independently of the
redundancy advisory. A `SET` that only assigns a trigger-maintained column
still updates the conflicting row. `OVERRIDING USER VALUE`,
`ON CONFLICT (column) WHERE ...`,
conditional `DO UPDATE ... WHERE ...`, `ON CONFLICT DO NOTHING`, composite,
expression, or operator-class conflict targets, non-plain `VALUES` or `SET`
expressions, different tables, different conflict columns, and different key
values leave the finding in place. As with direct `UPDATE` and `DELETE`
statements, `BEFORE INSERT` triggers that return `NULL` or rewrite `NEW`
columns (such as `NEW.id`) are not modelled.

A top-level helper with a straight-line body of static executor calls can accept
an executor parameter. Each invocation through a known transaction substitutes
its immutable argument bindings before comparing the helper's locked relations
and keys with the caller's writes. For example, `lockUser(tx.query, userId)`
requires an update or delete of that same user in `tx`; an update of a membership
does not satisfy the lock. Diagnostics name the helper and its lock line.
Straight-line helpers can call other supported local helpers. Their parameterized
locks and writes are composed at each invocation through a known transaction,
including a caller that passes its `TransactionQuery` parameter onward.
A helper with a plain executor parameter, no transaction caller, and no configured
executor-option path keeps its findings at the helper statement, including
line suppression there. Its ordinary callers do not multiply those findings.
A type annotation alone does not prove a transaction; unrelated transaction
instances and different keys remain distinct. Recursive helper cycles remain unresolved.

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
Each relation must have a matching write in the same transaction;
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
switch discriminant, which is evaluated in the outer environment. Namespace
blocks own their `var` bindings, including declarations inside nested blocks;
those bindings do not shadow an imported factory outside the namespace.
The rule can prove a row lock locally when its own SQL statement writes the
locked rows. For example, a CTE can select a column from rows with `FOR UPDATE`
and a `DELETE ... USING` can match the target to that CTE on the same column.
Use `IS NOT DISTINCT FROM` for a nullable join key. An equality join requires
a non-null key proven by the CTE predicate or by `schemaCatalogPath` with
complete coverage when the surrounding SQL leaves that schema evidence intact.
The rule checks for local proof before consulting the write set of a configured
transaction. Local proof can also establish that the locked rows are written
when transaction membership or other calls in the transaction are unresolved.
Within one SQL statement, repeated PostgreSQL parameter numbers such as `$1`
identify the same parameter. The rule cannot establish a matching key from
`?` or `:name` placeholders.

Set `transactionExecutorTypeNames: [TransactionQuery]` to match an awaited
lock and a later unconditional, awaited write on the same parameter in the
same function. SQL parameter arrays and trusted SQL tag interpolations can
prove matching keys when untransformed SQL values refer to the same stable primitive binding. This option
also selects those names as executor types and defaults to an empty list. Use names
imported directly from the configured executor module that exclusively represent
transaction executors. The proof requires a simple, stable parameter with a
direct type annotation. A pool-or-transaction union retains unresolved
membership. A lock without a proven local write remains unanalyzable because
the caller may perform the write after the function returns.

With an import specifier, executor names
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
Only complete catalogs contribute trigger-derived write evidence and non-null
CTE join-key evidence.

`row-lock-unwritten-relation`, `table-lock`, `constant-advisory-lock`, and
`row-lock-outside-transaction` default to `advise`. `share-lock-then-write`
and `redundant-prelock` default to `ignore`. Set `share-lock-then-write`
to `advise` or `fail`, or set `redundant-prelock` to `advise`, to enable
the corresponding check. [Default check levels](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/postgres_lock_written_rows_only.rs#L14-L38), [level names](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/postgres_lock_written_rows_only/policy.rs#L3-L10), and [pre-lock level validation](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/postgres_lock_written_rows_only/policy.rs#L41-L44).

`allow` defaults to `[]`; selectors are optional `relation`, `key`, `file`,
and `line`, with a required
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
lock ordering across transactions. Both sequence checks require consistent relation
qualification. If one statement uses a schema-qualified name and the other uses
the same unqualified name, the pair follows the configured unresolved SQL policy.
Key equality is checked only when provable; unresolved inputs follow the selected
policy rather than guessed equivalence.

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

The rule reads SQL from executor calls in application source. Source text
is decoded lossily, so a stray byte alone does not drop a file. [Source selection](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/postgres_lock_written_rows_only/scan.rs#L35-L50).
