# `postgres-no-duplicate-keyed-lookups`

Opt-in and off by default. Reports PostgreSQL statements that look up the same
target relation more than once on the same normalized key expression within a
single query — whether through repeated joins, repeated correlated scalar
subqueries, or a mix of both.

## Why and when

Repeated joins and independently evaluated correlated scalar subqueries can
probe the same table on the same key. Those probes can duplicate I/O and CPU
work.

Enable this rule on query-heavy services and reporting SQL where queries evolve
over time by tacking on extra `LEFT JOIN` clauses or inline scalar subqueries
for additional columns from an already-fetched relation.

## What it catches

Within a single `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `MERGE`, or `WITH` /
data-modifying CTE query, this rule flags a second or subsequent keyed lookup of
the same target relation when the normalized lookup key matches an earlier
lookup in the same scope. It also checks queries in views, `CREATE TABLE AS`,
`DECLARE CURSOR`, and `COPY (query) TO`. It covers `ON CONFLICT DO UPDATE`
expressions, referenced window definitions, and subqueries in ordering and
row-limit clauses.

- Repeated correlated scalar subqueries on the same relation and key, including
  unqualified inner key columns (`WHERE id = o.account_id`) and differing table
  aliases (`accounts a1` vs `accounts a2`).
- Repeated `JOIN` clauses (`INNER`, `LEFT`, `RIGHT`, `FULL`, `USING`) against
  the same relation on the same key, regardless of equality operand order
  (`a.id = o.account_id` vs `o.account_id = a.id`). This comparison
  assumes the join key uses commutative `=` semantics. For a chained
  `USING` join, both inputs of the preceding merge must have a provably
  identical column type, such as two aliases of the same relation. A join
  that can extend a binding with NULL starts a new lookup history for that
  binding.
- A `JOIN` combined with a correlated scalar subquery against the same relation
  and key.
- Equivalent PostgreSQL cast spellings for proven built-in values
  (`'active'::text` and `CAST('active' AS text)`).

The rule stays silent when the lookups are not proven to read the same row:

- Different outer key expressions (`o.buyer_id` vs `o.seller_id`) or different
  casts (`'active'::text` vs `'active'::varchar`).
- Different additional filter predicates (such as `addr.kind = 'billing'` vs
  `addr.kind = 'shipping'`).
- Scalar subqueries ordered by a bare name or ordinal position. Use a
  qualified column reference such as `ORDER BY accounts.priority` to identify
  the input column explicitly. Omitted sort direction and null placement
  use PostgreSQL defaults. Sort operators in `ORDER BY ... USING` are
  excluded.
- A lookup paired with a correlated aggregate (`COUNT(*)`, `SUM(...)`, etc.) or
  an `EXISTS (...)` test.
- Mutually exclusive `CASE WHEN ... THEN (SELECT ...) ELSE (SELECT ...) END`
  branches. A lookup in an arm can still repeat one in an independent
  expression. Inputs to recognized aggregates can repeat across arms because
  PostgreSQL evaluates the aggregates before choosing an arm. Aggregate
  arguments share lookup history only when their filter expressions match and
  the rule can prove the filter is repeatable.
- Optional default expressions in `lag` and `lead` window calls. PostgreSQL
  evaluates them only when the requested row falls outside the partition.
- Volatile key expressions (UUID generators such as `gen_random_uuid()`,
  `random()`, `clock_timestamp()`, sequence-advancing functions), user-defined
  or unrecognized functions in keys or scalar projections, unresolved table
  column alias lists, distinct `LIMIT` clauses, or ambiguous expressions.
- Function calls require a vetted built-in qualified with `pg_catalog` or
  PostgreSQL syntax such as `COALESCE`. Function arguments and cast inputs
  need known built-in types, and casts also need known built-in target types.
  The rule skips function arguments and casts of columns or parameters whose
  types it cannot establish. It also skips operators in compound expressions
  when their signatures are unknown, including the comparisons used by
  `IS DISTINCT FROM`, `IN`, `BETWEEN`, and pattern matching. Signature matching
  assumes the standard PostgreSQL operator definitions.
- Scalar projections with `CASE` branches that require unproven conversions
  or an unqualified comma-style `SUBSTRING` call.
- Sampled table scans, `ONLY` table scans, or scalar subqueries that lock rows.

PostgreSQL can reuse a select-list expression in `DISTINCT ON`, `GROUP BY`
(including positions and grouping sets), `ORDER BY`, and window partition and ordering
clauses. Identical named and inline window specifications can share their
frame bounds. An aggregate can also reuse an argument in its ordering clause. The
rule counts an identical reused expression once. A matching grouping
expression is also reused in projections, `HAVING`, `ORDER BY`, and window
function arguments. Ordinary aggregate inputs are evaluated before grouping.

The rule recognizes PostgreSQL 18's default `OLD` and `NEW` qualifiers in
`RETURNING`. It distinguishes pre-change and post-change rows and respects
table aliases that mask the default qualifiers.

Findings point at the duplicate join or scalar subquery and name the line of the
first lookup.

## Options and defaults

A project can configure `postgres.schemaCatalogPath` for other rules. This
rule uses the project executor and SQL scope settings.

This rule is off by default and runs only when explicitly configured in
`.no-mistakes.yml`. Unknown option keys fail early during configuration
validation (exit 2).

```yaml
rules:
  - rule: postgres-no-duplicate-keyed-lookups
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      sqlInclude: ["queries/**/*.sql"]
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
      unanalyzable: report
```

- `include` / `exclude`: glob filters for embedded TS/JS source files.
- `sqlInclude`: defaults to `[]`, so standalone `.sql` files are not scanned
  unless explicitly matched.
- `importSpecifier` / `executorNames` / `extraExecutorNames` /
  `excludeExecutorNames` / `executorFactoryNames` / `executorTypeNames` /
  `executorOptionProperties` / `trustedSqlTags`: standard PostgreSQL executor
  selection. Set `importSpecifier` (or `executorNames`) for embedded TS/JS
  queries. To scan only standalone `.sql` files selected by `sqlInclude`, set
  `executorNames: []` with `importSpecifier` unset (or another executor scope
  that selects no embedded calls); a nonempty `importSpecifier` with
  `executorNames: []` retains default executor selection. Setting both
  `executorNames` and `extraExecutorNames` is a configuration error. See
  [executor selection](../configuration/rules.md#postgresql-executor-selection).
- `unanalyzable`: shared uncertainty policy (`report`, `advise`, or `ignore`;
  default `report`). `report` fails on dynamically constructed or unparseable
  SQL, `advise` emits a non-failing advisory, and `ignore` omits residual
  diagnostics. Complete statements can still produce findings when another
  statement in the file cannot be parsed. Legacy `unanalyzableSql` (`fail`, `advise`, `ignore`) is accepted
  as an alias when `unanalyzable` is not set.

A `projects: [<name>]` entry inherits
[`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when
that block is configured. The rule rejects `reportUnmatchedExecutorNames`.

## Valid example

Combine the lookups into a single join and project every needed column from it:

```sql
-- queries/orders.sql
SELECT
  o.id,
  a.display_name AS account_name,
  a.tier AS account_tier
FROM orders o
LEFT JOIN accounts a ON a.id = o.account_id;
```

## Counterexample

Two scalar subqueries looking up the same `accounts` row on `o.account_id`:

```sql
-- queries/orders.sql
SELECT
  o.id,
  (SELECT display_name FROM accounts WHERE id = o.account_id) AS account_name,
  (SELECT tier FROM accounts WHERE id = o.account_id) AS account_tier
FROM orders o;
```

Two joins against the same `accounts` row on `o.account_id`:

```sql
-- queries/orders.sql
SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = o.account_id
LEFT JOIN accounts a2 ON o.account_id = a2.id;
```

## Fix

Verify that the lookup key identifies at most one row (or otherwise preserves
query cardinality and behavior) before consolidating repeated joins or
correlated scalar subqueries into a single join projecting every needed column.
Preserve the original filtering semantics: use `INNER JOIN` when the original
query excluded rows without a match, and `LEFT JOIN` when unmatched rows should
remain in the result. For `RIGHT JOIN` and `FULL JOIN`, also verify which
unmatched target rows survive and how many copies the original query returns.

## Suppression

Use the standard rule suppression directives in both `.sql` (`--`) and TS/JS
(`//`) files:

- `-- no-mistakes-disable-next-line postgres-no-duplicate-keyed-lookups`
- `-- no-mistakes-disable-line postgres-no-duplicate-keyed-lookups`
- `-- no-mistakes-disable-file postgres-no-duplicate-keyed-lookups`

For embedded SQL in TS/JS, use `-- no-mistakes-disable-next-line` on a line
inside the SQL template, or JS/TS `// no-mistakes-disable-line`,
`// no-mistakes-disable-next-line`, or `// no-mistakes-disable-file` comments.

## Related rules

- [`postgres-no-offset`](postgres-no-offset.md) flags `OFFSET` pagination in
  executed PostgreSQL SQL.
- [`postgres-grouping-not-aggregation`](postgres-grouping-not-aggregation.md)
  advises on grouping used for deduplication or existence.
- [`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) flags configured
  anti-pattern SQL shapes.
