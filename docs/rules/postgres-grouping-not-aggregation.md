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
Messages include a column to distinguish separate SELECT sites on one source line.
For embedded SQL, this is the source column when provenance is available.
This does not add a separate severity or suppression syntax per shape.

All shapes require a configured schema catalog. With no `schemaCatalogPath`,
the rule stays silent for every shape. It uses primary keys, unique constraints,
and unique indexes. Partial indexes require an exact normalized predicate
conjunct in the query (with the supported equality/non-null evidence); this is
not a general predicate-implication solver. For example, `WHERE deleted_at IS
NULL` can prove that partial predicate, while `WHERE deleted_at IS NULL OR
other_condition` cannot. Unknown relations, ambiguous ownership, incomplete SQL,
and unsupported expressions are left silent when the necessary proof is unavailable.

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
index requires its predicate to be proven by an exact supported query conjunct.
Nullable unique keys also need non-null proof unless the index declares `NULLS
NOT DISTINCT`; ordinary uniqueness permits repeated NULL values.
For `COUNT(DISTINCT expression)`, repeated NULL values do not prevent dropping
`DISTINCT`: `COUNT(expression)` also ignores them. Retain the counted expression
and any aggregate filter.

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
multiplied side supplies projected or aggregated values, including ordering
expressions. The join predicate must cover the derived distinct projection;
otherwise the join can still multiply rows.

For membership operators, the safe change is to remove `DISTINCT` while keeping
the original `IN`, `NOT IN`, `ANY`, or `ALL` operator. Do not turn `NOT IN` or
`<> ALL` into `NOT EXISTS` unless non-null keys or explicit guards preserve
SQL's NULL behavior. Duplicate removal alone does not remove NULL values.
Suppression: retain a measured grouped filter with the normal disable comment.

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

For grouped `MAX` tests, preserve both the original absence condition and
three-valued comparisons. When the timestamp column is proven `NOT NULL`, this:

```sql
SELECT r.post_id FROM post_reads r GROUP BY r.post_id
HAVING NOT EXISTS (
  SELECT 1 FROM post_reads x WHERE x.post_id = r.post_id
) OR MAX(r.created_at) < $1;
```

can use a guarded anti-existence test:

```sql
SELECT r.post_id FROM post_reads r GROUP BY r.post_id
HAVING NOT EXISTS (
  SELECT 1 FROM post_reads x WHERE x.post_id = r.post_id
) OR ($1 IS NOT NULL AND NOT EXISTS (
  SELECT 1 FROM post_reads x
  WHERE x.post_id = r.post_id AND x.created_at >= $1
));
```

Keep the original `NOT EXISTS` disjunct, especially for nullable grouping keys.
The threshold guard is necessary because `MAX(...) < NULL` is unknown, while
`NOT EXISTS (... >= NULL)` is true. For a catalog-known nullable input, also require `EXISTS` for rows with a
non-NULL input: an all-NULL group must remain excluded. Both new probes read the
physical outer table and correlate every grouping key, independently of any
extra filters in the original absence subquery. Preserve that original disjunct.
For example, replace only the MAX disjunct with:

```sql
$1 IS NOT NULL
AND EXISTS (SELECT 1 FROM live_topics x
            WHERE x.topic_id = t.topic_id AND x.deleted_at IS NOT NULL)
AND NOT EXISTS (SELECT 1 FROM live_topics x
                WHERE x.topic_id = t.topic_id AND x.deleted_at >= $1)
```

These guards preserve the HAVING rowset; they need not preserve UNKNOWN versus
FALSE as a standalone boolean value. This additional null-aware guidance requires
a catalog-known input column; the literal anti-threshold proof does not require
column nullability metadata.
If the original absence subquery already tests `created_at >= $1`, retain that
original `NOT EXISTS` and remove the redundant `MAX` disjunct instead; its NULL
behavior is already represented by the existing predicate.

A count returned to the caller remains valid. Suppression: use a normal disable
comment when an explain-analyze scenario and benchmark justify the aggregate.

## Counterexample

These named files use the configuration above. The catalog's composite primary
key and equality pin prove that `DISTINCT` is redundant:

```json
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "post_topics": {
      "columns": {
        "post_id": {"dataType": "uuid", "nullable": false},
        "topic_id": {"dataType": "uuid", "nullable": false}
      },
      "primaryKey": {"columns": ["post_id", "topic_id"]}
    }
  }
}
```

```sql
-- db/queries/post-topics.sql
SELECT DISTINCT post_id FROM post_topics WHERE topic_id = $1;
```

Fix: remove `DISTINCT`; the remaining primary-key column is already unique in
this result. This example produces an advisory whose `target` is
`distinct-over-unique-key` and does not fail the run.

For dynamic candidate terms, retain caller input validation and the original
empty/NULL-list behavior: zero ANDed `EXISTS` predicates can be vacuously true
while an empty candidate `GROUP BY` produces no rows.

## Valid example

Real aggregation is valid. These named files provide its catalog and selected
query using the same configuration:

```json
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "ratings": {
      "columns": {
        "topic_id": {"dataType": "uuid", "nullable": false},
        "score": {"dataType": "integer", "nullable": false}
      }
    }
  }
}
```

```sql
-- db/queries/ratings.sql
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
| `sqlInclude` | shared PostgreSQL scope | Explicit SQL paths/globs; project PostgreSQL scope can supply these. An empty list uses the shared PostgreSQL fallback within the selected application scope. |
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
