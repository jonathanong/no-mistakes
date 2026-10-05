# `postgres-lock-ordering`

Flags executed PostgreSQL SQL that takes a `FOR UPDATE` lock with a multi-row
predicate (`IN` or `= ANY`) but no `ORDER BY` and no `SKIP LOCKED`. Two
transactions that lock the same rows in opposite order can deadlock (ABBA).

The rule consumes request-prepared PostgreSQL embedded-SQL facts and
`extract_locking_select_metadata`. It does not re-parse TypeScript in the rule
engine.

```yaml
rules:
  - rule: postgres-lock-ordering
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
      safeDirective: deadlock-safe
      schemaCatalogPath: db/schema.json
```

`importSpecifier` has no default. `executorNames` defaults to `query`, `read`, and `write` only when `importSpecifier` is configured. `safeDirective` defaults to `deadlock-safe`.
`schemaCatalogPath` is optional; when present it must be a repository-relative
catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md).

Counterexample: `query(\`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE\`)`without`ORDER BY`or`SKIP LOCKED`. Unparseable `FOR UPDATE` SQL is a
separate diagnostic so lock statements stay parseable.

```ts
import { query } from "@example/db";

export function lockRows(ids: string[]) {
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
```

Fix: add `ORDER BY` so every locker visits rows in the same order, add
`SKIP LOCKED` when skipping already-locked rows is correct, or put
`/* deadlock-safe: ... */` or `-- deadlock-safe` in a comment within 200
characters before the call when a unique key makes the lock single-row.

```ts
query(`SELECT * FROM t WHERE id = ANY($1) ORDER BY id FOR UPDATE`);
query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE SKIP LOCKED`);

/* deadlock-safe: single row via unique key */
query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
```

Use `no-mistakes-disable-next-line postgres-lock-ordering` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule for worker or request code that locks several rows by ID, where
concurrent transactions can visit the same set in different orders.

## What it catches/requires

A multi-row `FOR UPDATE` query using `IN` or `= ANY` must order its rows or use
`SKIP LOCKED`. The embedded SQL must be statically recoverable; unparseable
statements receive a separate diagnostic.

`FOR NO KEY UPDATE` is checked exactly like `FOR UPDATE`, with a single target or
an `OF` list: lock ordering depends on which rows are locked and in what order,
not on the lock strength. Shared locks (`FOR SHARE` and `FOR KEY SHARE`) are
accepted by the parser but are not checked, since shared lockers do not wait on
each other and cannot form this ABBA cycle.

With `schemaCatalogPath`, an ordinary multi-row lock must also begin its
`ORDER BY` with the ordered expression keys of one valid, ready, non-partial
btree unique index for every locked base table. Leading columns of that key
that are pinned for the locked relation are already fixed on every row, so they
are not part of the required prefix. A pin is a top-level `AND` equality in
`WHERE` or an inner join's `ON` to a literal, a `$n` placeholder, or a recovered
template interpolation — the same columns `pinned_columns` records. Only a
leading prefix is dropped: pinning `guid` does not remove it from
`UNIQUE (host_id, guid)` while `host_id` is still required. `ORDER BY` must
begin with the remaining columns, with the same expression, ascending
direction, and nulls semantics, and a qualifier must still name the locked
relation. A key whose columns are all pinned needs no `ORDER BY` prefix from
that key. Equalities inside `OR` or `NOT`, and pins on a different relation,
do not count.

```ts
// feed_items UNIQUE (host_id, guid): every locked row shares host_id.
query(
  `SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2) ORDER BY guid FOR UPDATE`,
);
```

`FOR UPDATE OF alias` limits
the requirement to that resolved relation; a lock with no `OF` clause over joins or
derived relations, or an unresolved `OF` target, fails closed rather than silently
checking only the first `FROM` table. A comma-separated list such as `FOR UPDATE OF a, o` (with an
optional `NOWAIT` or `SKIP LOCKED` after it) is one locking clause covering every
listed relation, checked exactly like `FOR UPDATE OF a FOR UPDATE OF o`: with a
catalog, every resolved relation needs its key prefix, and one unresolved name
fails closed. This makes reader lock order match the catalog-backed writer
order instead of accepting an unrelated deterministic sort. `SKIP LOCKED`
remains an alternative because it avoids waiting for an already-held row lock.

### Unique-key lookups with an `IN` / `= ANY` filter

With `schemaCatalogPath`, an `IN` or `= ANY` predicate does not make the lock
multi-row when every locked table has a catalog unique key (valid, ready,
non-partial, immediate, plain columns) whose columns are all pinned by top-level
`AND` equalities of `WHERE` or of an inner join's `ON`. A column is pinned by an
equality to a literal, a `$n` placeholder, a recovered template interpolation
(`${id}`; user-authored text that only spells the marker is a column), or a column
of a relation that is already single-row, repeated until nothing new is proven (so
`JOIN grants g ON g.id = code.grant_id` is single-row once `code` is pinned by its
unique `token_hash`). The unique key bounds the statement to one row per locked
table, so a filter such as
`status IN ('open', 'held')` or `callback_url = ANY(a.callback_urls)` can only
narrow it. Equalities inside `OR` or `NOT`, an equality in an outer join's `ON`, an unqualified column in a join, a pin
on a table the lock does not name, or a self-join do not pin a column. A partially pinned composite key does not
prove a single-row lock: with no `ORDER BY`, or with one that does not start with the remaining catalog key, it still
fails closed. Without a catalog no key is known to be unique,
so the original check applies.

```ts
// orders.id is the primary key: one row, so the IN list is only a filter.
query(`SELECT id FROM orders WHERE id = $1 AND status IN ('open', 'held') FOR UPDATE`);
```

### Derived relations and interpolated relation names

`FOR UPDATE OF alias` that names one base table is checked against that table
alone: a `LATERAL` subquery, derived table, or table function elsewhere in `FROM`
is not locked and does not change the locked table's `ORDER BY` key prefix. Without
an `OF` clause every `FROM` item is locked, so a derived relation still fails
closed, as does an `OF` target that is a derived relation.

A relation name built by interpolation (`FROM ${table}`) cannot be looked up in the
catalog. Instead of the key-prefix message, the rule reports a distinct
"relation name is interpolated" diagnostic. It is suppressed the same way as other
findings: the safe directive comment, `no-mistakes-disable-next-line`, or writing
the relation name literally. The rule does not resolve `const` tuples or string
unions into their members.

## Options and defaults

`include` and `exclude` select source files. `importSpecifier` has no default, `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured, and
`safeDirective` defaults to `deadlock-safe`. `schemaCatalogPath` defaults to
unset; set it to a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md) to enable the catalog
exact-prefix requirement.

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
| `executorFactoryNames` | Empty | Named imports (from `importSpecifier`, or any module when it is empty) whose call result bound with `const`, `let`, `using`, or `await using` is an executor inside the declaring block. |
| `executorTypeNames` | Empty | Imported type names (`import type` or inline `type` specifiers) whose annotated parameters, including optional and inline-destructured ones, are executors inside the declaring function. |
| `reportUnmatchedExecutorNames` | `false` | Opt in to one finding per `executorFactoryNames` / `executorTypeNames` entry that none of the files this rule scans imports from `importSpecifier` or its subpaths (a typo or a wrong module silently disables checking). Reported against the config file, so line suppression does not apply; remove the entry or leave this off to silence it. Rust rule only. See [Reporting unmatched names](../migrations/explicit-postgres-executors.md#reporting-unmatched-names). |

`executorFactoryNames` and `executorTypeNames` add scoped executors: `tx` in
`await using tx = await openTransaction()` or `run` in `run: TxExecutor` is scanned
only inside the block or function that declares it, and a same-named identifier
elsewhere is not. Matching is by configuration only, both default to empty, and
neither counts as executor selection: still set `importSpecifier` or `executorNames`.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`; `executorNames` still requires the exact module.

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```ts
query(`SELECT * FROM jobs WHERE id = ANY($1) ORDER BY id FOR UPDATE`);
```

## Counterexample

```ts
query(`SELECT * FROM jobs WHERE id = ANY($1) FOR UPDATE`);
```

## Fix

Add deterministic `ORDER BY`, use `SKIP LOCKED` for a work queue, or document
why a unique-key lookup is effectively single-row with the configured safe
directive.

## Suppression

Use `no-mistakes-disable-next-line postgres-lock-ordering` or
`no-mistakes-disable-line`; reserve the file directive for a file whose locking
contract is enforced elsewhere.

## Related rules

[`postgres-no-offset`](postgres-no-offset.md) covers inefficient pagination;
[`postgres-require-query-annotation`](postgres-require-query-annotation.md)
ensures lock statements remain identifiable in query logs; and
[`postgres-conflict-ordering`](postgres-conflict-ordering.md) requires shared
multi-row conflict writers to use the same catalog order.

## Generated catalogs

[`postgres catalog`](../cli/postgres.md) observes PostgreSQL directly. This rule
accepts both its `complete` and `ordering` coverage, and reports identical findings
for either. Ordering coverage emits only independent ordering facts. It retains invalid/not-ready/non-live index state;
those indexes cannot prove order. Unsupported custom operator classes/collations remain
in the JSON but fail closed for canonical proof. Deferrable unique indexes cannot
be conflict arbiters, although supported valid keys can still prove lock order.
An ordering catalog declares its coverage and cannot satisfy rules requiring
complete schema metadata; a complete catalog satisfies every catalog rule.
