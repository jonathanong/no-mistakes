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
      schemaCatalogPath: db/schema.json
      include: ["backend/**/*.ts"]
      sqlInclude: ["backend/queries/**/*.sql"]
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
      unanalyzableSql: fail
      safeDirective: deadlock-safe
```

`schemaCatalogPath` is required and must name a repository-relative catalog
generated with [`no-mistakes postgres catalog`](../cli/postgres.md). The catalog, not a
lexical sort, supplies an index's expression and key order. `sqlInclude` is
opt-in (default `[]`) and adds static `.sql` query files. Typed executor calls
require an explicit `importSpecifier` or `executorNames`. Trusted tags recover SQL
text only after the executor has been selected.
`importSpecifier` defaults to empty; when configured without explicit names,
`executorNames` defaults to `[query, read, write]`. Transaction helpers and
`.query(...)` calls use the shared typed-executor facts.

`unanalyzableSql` defaults to `fail`; set it to `ignore` only while an explicit
exception is being removed. `safeDirective` defaults to `deadlock-safe`.

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
deliberately not guessed. Normalization ignores case, whitespace and redundant
parentheses around `AND`/`OR` operands, so `deleted_at IS NULL AND kind IS NOT NULL`
matches the catalog's `((deleted_at IS NULL) AND (kind IS NOT NULL))`. Parentheses that
change precedence, such as `a AND (b OR c)`, and the order of conjuncts stay significant.

### Single-row sources and constant keys

A source that provably yields at most one row has no second writer order to disagree
with, so it needs no `ORDER BY`. The rule accepts only these shapes:

- a `SELECT` without `FROM` whose select list holds literals, bound parameters, casts,
  scalar subqueries and a short list of scalar functions (`lower`, `upper`, `coalesce`,
  `nullif`, `concat`, `now`, `gen_random_uuid`, ...). A set-returning function such as
  `unnest` or `generate_series` can expand it, so it is not single-row;
- a `SELECT` from one plain table (no join, CTE shadowing or table function) whose
  top-level `AND` conjuncts equate every column of one catalog unique key (valid, ready,
  immediate, non-partial) to a literal or bound parameter;
- a literal `LIMIT 0`/`LIMIT 1` or `FETCH FIRST [1] ROW ONLY`.

Anything else, including `OR` filters, partial unique indexes, a partly pinned composite key
and `LIMIT $1`, stays multi-row and fail-closed. A positional `ORDER BY 1, 2` maps to the
select list. A key that is a literal or bound parameter is the same in every row, so the
`ORDER BY` may include it or omit it; the remaining keys must still lead in catalog order.

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
row order. An opaque configured executor argument (`read(assembleWriter())`,
`write(...args)`, or `query(...args)` by default), as well as an opaque member
call (`client.query(assembleWriter())`), also fails closed by default: there is no recoverable
statement, so a zero-finding run cannot mean the writer was checked.
Statement-level `sql-template-strings` mutation chains — initialize a bound
statement, `append` recoverable fragments, then `read`/`write` it — are
analyzed when those fragments are static, including a same-file helper whose
body is a straight-line `const`/`let` plus `.append` chain ending in
`return` (`write(buildExpireQuery())`). Complete `WITH` text is classified by
the final top-level `SELECT`/`INSERT`/`UPDATE`/`DELETE`/`MERGE` after
balanced CTE definitions; catalog ordering runs only when that recovered
statement is a full `INSERT`/`UPSERT`. An `INSERT` that appears only inside
a CTE does not make a final `UPDATE` an insert. Incomplete `WITH` prefixes
and opaque or imported helpers stay fail-closed. Kind-only recovery never
fabricates SQL. Conditional static appends keep the recovered base SQL and
classify as dynamic: recovered non-`INSERT` SELECT/UPDATE stays outside this
rule, while recovered `INSERT` fails closed rather than treating a
branch-only `ORDER BY` as always present. Sequential recovered
`INSERT … ON CONFLICT` gets the ordinary catalog ordering check. Unbound,
spread, or otherwise opaque append arguments preserve a recovered leading
statement as dynamic: leading `INSERT` stays fail-closed, while leading
`SELECT`/`UPDATE` remains outside this rule. A wholly opaque executor argument
still fails closed. Make the statement static, use `unanalyzableSql: ignore`
for a temporary scoped rollout exception, or add a nearby SQL/comment directive such as
`/* deadlock-safe: single ordered source */` only when the ordering is
enforced outside the analyzable statement. Recovered dynamic SQL that is
not an `INSERT` is still ignored.

Use `no-mistakes-disable-next-line postgres-conflict-ordering` or
`no-mistakes-disable-line` for a one-off. Prefer repairing the writer or a
specific `safeDirective` explanation over a file-level suppression.

## Related rules

[`postgres-lock-ordering`](postgres-lock-ordering.md) applies the same optional
catalog key-prefix requirement to multi-row `FOR UPDATE` readers.

## Generated catalogs

[`postgres catalog`](../cli/postgres.md) observes PostgreSQL directly. This rule
accepts both its `complete` and `ordering` coverage, and reports identical findings
for either. Ordering coverage emits only independent ordering facts. It retains invalid/not-ready/non-live index state;
those indexes cannot prove order. Unsupported custom operator classes/collations remain
in the JSON but fail closed for canonical proof. Deferrable unique indexes cannot
be conflict arbiters, although supported valid keys can still prove lock order.
An ordering catalog declares its coverage and cannot satisfy rules requiring
complete schema metadata; a complete catalog satisfies every catalog rule.
