# `postgres-conflict-ordering`

Requires every analyzable multi-row `INSERT … ON CONFLICT` to acquire unique-index
conflict locks in the committed schema catalog's canonical key order. It prevents
two writers from visiting the same conflicting rows in different orders.

## Why and when

Enable this for production writers that can insert several rows per statement
and may run concurrently, especially queue consumers, retry paths, and
backfills. It complements replay safety: `ON CONFLICT` can avoid duplicate rows
while still deadlocking if concurrent statements acquire the same index locks
in incompatible orders.

## Options and defaults

```yaml
rules:
  - rule: postgres-conflict-ordering
    scope: repository
    options:
      schemaCatalogPath: backend/data-stores/psql/schema-snapshot/schema.json
      include: ["backend/**/*.ts"]
      sqlInclude: ["backend/queries/**/*.sql"]
      importSpecifier: "@data-stores/psql"
      executorNames: [query, read, write]
      unanalyzableSql: fail
      safeDirective: deadlock-safe
```

`schemaCatalogPath` is required and must name a repository-relative PostgreSQL
schema snapshot with `formatVersion: 2`. The snapshot, not a
lexical sort, supplies an index's expression and key order. `sqlInclude` is
opt-in (default `[]`) and adds static `.sql` query files; typed executor calls
are always considered. `importSpecifier` defaults to `@data-stores/psql` and
`executorNames` to `[query, read, write]`. Transaction helpers and `.query(...)`
calls use the shared typed-executor facts.

`unanalyzableSql` defaults to `fail`; set it to `ignore` only while an explicit
exception is being removed. `safeDirective` defaults to `deadlock-safe`.

## What it catches

For a potentially multi-row `INSERT … SELECT … ON CONFLICT`, the rule:

1. rejects `ON CONFLICT DO NOTHING` when its target is omitted, because no one
   arbiter can be selected;
2. resolves the column/expression target plus an optional partial-index
   predicate against valid, ready btree unique indexes in the catalog;
3. rejects an unresolved or differently ordered set of inferred indexes;
4. requires the written conflict target itself to match the selected catalog key
   sequence; and
5. requires the source `ORDER BY` to begin with that same key sequence after
   mapping inserted columns to their `SELECT` expressions.

The comparison parses SQL expressions: for example, an index key
`lower(category_text)` maps to `lower(input.category_text)` without treating
`lower` as a column. It also resolves a top-level `SELECT` alias, so
`SELECT input.id AS conflict_id ... ORDER BY conflict_id` is accepted. Only an
exact normalized partial-index predicate is accepted; logical implication is
deliberately not guessed.

## Valid example

```sql
INSERT INTO rss_feed_item_categories (rss_feed_item_id, category_text)
SELECT input.rss_feed_item_id, input.category_text
FROM unnest($1::uuid[], $2::text[]) AS input(rss_feed_item_id, category_text)
ORDER BY input.rss_feed_item_id, lower(input.category_text), input.category_text
ON CONFLICT (rss_feed_item_id, (lower(category_text))) DO NOTHING;
```

Here the catalog's unique index begins with `rss_feed_item_id` and then
`lower(category_text)`. The final `category_text` tie-breaker is allowed after
the required key prefix.

## Counterexample and fix

This target identifies the same unique-key set but reverses the lock order:

```sql
INSERT INTO edges (left_id, right_id)
SELECT input.left_id, input.right_id
FROM unnest($1::uuid[], $2::uuid[]) AS input(left_id, right_id)
ORDER BY input.left_id, input.right_id
ON CONFLICT (right_id, left_id) DO NOTHING;
```

Write both the target and source order in the catalog order:

```sql
ON CONFLICT (left_id, right_id) DO NOTHING
```

## Dynamic SQL and suppression

Recovered dynamic SQL whose static fragments identify an `INSERT` fails closed
by default because an interpolation can add or alter its conflict clause and
row order. An opaque executor argument (`query(assembleWriter())`,
`query(...args)`) also fails closed by default: there is no recoverable
statement, so a zero-finding run cannot mean the writer was checked.
Statement-level `sql-template-strings` mutation chains — initialize a bound
statement, `append` recoverable fragments, then `read`/`write` it — are
analyzed when those fragments are static, including helper-produced
`SQLStatement` values. Conditional static appends keep the recovered base
SQL and classify as dynamic: recovered non-`INSERT` SELECT/UPDATE stays
outside this rule, while recovered `INSERT` fails closed rather than treating
a branch-only `ORDER BY` as always present. Sequential recovered
`INSERT … ON CONFLICT` gets the ordinary catalog ordering check. Unbound,
spread, or otherwise opaque append arguments stay fail-closed. Make the
statement static, use `unanalyzableSql: ignore` for a temporary scoped
rollout exception, or add a nearby SQL/comment directive such as
`/* deadlock-safe: single ordered source */` only when the ordering is
enforced outside the analyzable statement. Recovered dynamic SQL that is
not an `INSERT` is still ignored.

Use `no-mistakes-disable-next-line postgres-conflict-ordering` or
`no-mistakes-disable-line` for a one-off. Prefer repairing the writer or a
specific `safeDirective` explanation over a file-level suppression.

## Related rules

[`postgres-lock-ordering`](postgres-lock-ordering.md) applies the same optional
catalog key-prefix requirement to multi-row `FOR UPDATE` readers.
