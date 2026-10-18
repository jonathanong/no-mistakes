# Lock Ordering Proof

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
`UNIQUE (host_id, guid)` while `host_id` is still required. `ORDER BY` may
keep those pinned columns. It must begin with either the full key or the key
after that leading prefix is dropped, with the same expression, ascending
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

### Unanalyzable SQL

**Behavior change:** this rule previously skipped executor calls whose SQL text
could not be recovered. With the default `unanalyzableSql: fail`, a dynamic
executor call without complete alternatives or recovered SQL text (an opaque
`query(sql)` argument, a `query(cond ? a : b)` choice whose arms cannot all be
recovered, an unresolved builder) is reported with target
`unanalyzable`, because that SQL could take a `FOR UPDATE` lock whose row order
cannot be checked. See the
[migration note](../migrations/postgres-unanalyzable-sql.md).

To stay conservative without flagging every dynamic read, dynamic SQL whose
recovered text exists is handled as before: when it contains `FOR UPDATE` or
`FOR NO KEY UPDATE` it gets the ordinary lock-ordering checks (interpolations
are binds or interpolated relation names), and when it contains no exclusive
lock clause it is treated as non-locking. A lock clause appended only through an
opaque tail is therefore not detected; keep lock clauses in the recovered SQL.
Calls with recovered SQL never produce `unanalyzable` findings here.

```ts
import { query } from "@example/db";

// Counterexample: reported as unanalyzable by default.
export function lockAny(sql: string, ids: string[]) {
  return query(sql, [ids]);
}
```

Fix: pass a SQL literal or a configured trusted tagged template so the lock can
be checked, add the configured safe directive (`/* deadlock-safe: ... */`) within
200 bytes before the call when callers are known to be safe, suppress the
line with `no-mistakes-disable-next-line postgres-lock-ordering`, or set
`unanalyzableSql: ignore` to restore the earlier skip.

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

