# `postgres-grouping-not-aggregation`

## Why and when

Use this opt-in advisory when reviewing grouping used to repair duplicate input
or test existence. Unnecessary grouping can add a sort or hash aggregation and
hide a direct `EXISTS` or `NOT EXISTS` filter. Compare plans and benchmarks before
rewriting: a proven faster grouped query can use a normal suppression.

This rule cannot fail a run. Findings appear in `advisories`, including when
severity is omitted. Only explicit `severity: advisory` is accepted; `error`,
`fail`, and `options.fail` are rejected. Configuration errors remain errors.

```yaml
rules:
  - rule: postgres-grouping-not-aggregation
    scope: repository
    severity: advisory
    options:
      schemaCatalogPath: db/schema.json
      sqlInclude: [db/queries/**/*.sql]
      include: [src/**/*.ts]
      importSpecifier: db
      executorNames: [query]
```

## What it catches

The five stable message codes are carried in each finding's `target` field.
Consumers can filter advisory records by `rule` and `target` to select one shape.
This does not add a separate severity or suppression syntax per shape.

All shapes require a configured schema catalog. With no `schemaCatalogPath`,
the rule stays silent for every shape. It uses primary keys, unique constraints,
and unique indexes, including partial indexes only when the query implies their
predicate. Unknown relations, ambiguous ownership, incomplete SQL, and unsupported
expressions are left silent when the necessary proof is unavailable.

The rule analyzes selected raw SQL and statically recovered SQL passed to configured
executors, including template literals and `.append` fragments. It borrows the
request's prepared SQL, source locations, and catalog. It does not run SQL or
query a live database. See [catalog generation](../postgres-facts/generated-schema-catalogs.md).

### `all-of-candidate-pool`

Counterexample: an all-of filter aggregates a pool selected by `ANY` or a union.

```sql
SELECT item_id FROM tags WHERE tag_id = ANY($1)
GROUP BY item_id HAVING COUNT(DISTINCT tag_id) = $2;
```

Fix: drive from `items` and AND one existence predicate for each required term:

```sql
SELECT i.id FROM items i
WHERE EXISTS (SELECT 1 FROM tags t WHERE t.item_id = i.id AND t.tag_id = $1)
  AND EXISTS (SELECT 1 FROM tags t WHERE t.item_id = i.id AND t.tag_id = $2);
```

A real aggregate returned as a value is a counter-control, not an all-of filter.
Suppression: add `-- no-mistakes-disable-next-line postgres-grouping-not-aggregation`
before a measured grouped implementation whose plan and benchmark justify it.

### `distinct-over-unique-key`

Counterexample with primary key `(post_id, topic_id)`:

```sql
SELECT DISTINCT post_id FROM post_topics WHERE topic_id = $1;
```

Fix: drop `DISTINCT`; equality pins the other key column. The same proof can
apply to `DISTINCT ON`, grouping keys, or a distinct aggregate. A partial unique
index requires its predicate to be proven by the query.

```sql
SELECT post_id FROM post_topics WHERE topic_id = $1;
```

A latest-row `DISTINCT ON (topic_id) ORDER BY topic_id, ts DESC` over a non-unique
key remains valid. Suppression: use a normal disable comment only when measured
evidence supports retaining the redundant grouping.

### `distinct-over-union`

Counterexample: outer `DISTINCT` repeats the deduplication of an immediate union.

```sql
SELECT DISTINCT id FROM
(SELECT id FROM active_items UNION SELECT id FROM archived_items) p;
```

Fix: remove the outer `DISTINCT` when both operations deduplicate the same rows.

```sql
SELECT id FROM active_items UNION SELECT id FROM archived_items;
```

A `UNION ALL` whose duplicated rows need deduplication remains valid. Projecting
fewer columns than the union can also require distinct output. Recursive CTE
cycle guards remain silent. Suppression: retain a measured faster shape with a
normal disable comment and its plan/benchmark rationale.

### `distinct-join-filter`

Counterexample: a distinct derived relation is used only to filter driving rows.

```sql
SELECT i.id FROM items i
JOIN (SELECT DISTINCT item_id FROM tags) t ON t.item_id = i.id;
```

Fix: express the semi-join directly.

```sql
SELECT i.id FROM items i
WHERE EXISTS (SELECT 1 FROM tags t WHERE t.item_id = i.id);
```

The advisory also covers proven anti-join/existence consumers and distinct
aggregates repairing filter-only join fan-out. It stays silent when the
multiplied side supplies projected or aggregated values. Suppression: retain a
measured grouped filter with the normal disable comment.

### `aggregate-existence-test`

Counterexample: an aggregate is used only to test whether any rows exist.

```sql
SELECT i.id FROM items i
WHERE (SELECT COUNT(*) FROM tags t WHERE t.item_id = i.id) > 0;
```

Fix: use `EXISTS`; use `NOT EXISTS` for a zero-count test. Proven grouped
anti-existence/maximum-time filters can similarly use an anti-join.

```sql
SELECT i.id FROM items i
WHERE EXISTS (SELECT 1 FROM tags t WHERE t.item_id = i.id);
```

A count returned to the caller remains valid. Suppression: use a normal disable
comment when an explain-analyze scenario and benchmark justify the aggregate.

## Valid example

Real aggregation is valid:

```sql
SELECT topic_id, COUNT(*) AS n, SUM(score) AS score
FROM ratings GROUP BY topic_id;
```

So are legitimate duplicated-input deduplication and latest-row selection over
non-unique keys. The analyzer reports only the shapes it can prove.

## Options and defaults

| Option | Default | Meaning |
| --- | --- | --- |
| `schemaCatalogPath` | unset | Catalog JSON path relative to root; unset or empty disables every shape. |
| `include`, `exclude` | `[]` | Additional source include and exclude globs inside the selected application scope. |
| `sqlInclude` | shared PostgreSQL scope | Explicit SQL paths/globs; project PostgreSQL scope can supply these. Without a locked scope, an empty list uses the shared `**/*.sql` fallback. |
| `sqlIncludeLocked` | `false` | Resolved SQL scope is already fixed by the caller. |
| `importSpecifier` | empty | Module exporting configured executors. |
| `executorNames` | `query`, `read`, `write` | Replaces executor names; mutually exclusive with extras. |
| `extraExecutorNames` | unset | Adds names to the defaults. |
| `excludeExecutorNames` | `[]` | Removes names after replacement/defaults and extras. |
| `executorFactoryNames`, `executorTypeNames` | `[]` | Explicit executor factories and types. |
| `executorOptionProperties` | `[]` | Declared option properties carrying an executor. |
| `trustedSqlTags` | `[]` | Explicit `{module, name}` SQL tags. |
| `sqlLiteralHelpers` | `[]` | Configured literal-preserving SQL helpers. |

No allow list or fail option is supported. Public Node options are exported as
`PostgresGroupingNotAggregationOptions`; the target union is
`PostgresGroupingNotAggregationCode`.

## Suppression

Use `no-mistakes-disable-file`, `no-mistakes-disable-line`, or
`no-mistakes-disable-next-line` with this rule ID. SQL comments and embedded
source comments use the normal [suppression rules](../configuration/suppressions.md).
For example:

```sql
-- Explain-analyze scenario and benchmark show this grouping is faster.
-- no-mistakes-disable-next-line postgres-grouping-not-aggregation
SELECT DISTINCT post_id FROM post_topics WHERE topic_id = $1;
```

For embedded SQL, place the directive at the reported source line. There is no
shape-specific disable-comment suffix: filter `target` in programmatic consumers,
and use ordinary location-scoped directives for source exceptions.

## Related rules

- [`postgres-bounded-statements`](postgres-bounded-statements.md) checks row bounds.
- [`postgres-required-predicates`](postgres-required-predicates.md) checks required predicates.
- [`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) configures structural SQL policies.
