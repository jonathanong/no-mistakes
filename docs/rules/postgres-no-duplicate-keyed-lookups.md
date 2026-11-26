# `postgres-no-duplicate-keyed-lookups`

Opt-in and off by default. Reports PostgreSQL statements that look up the same
target relation more than once on the same normalized key expression within a
single query — whether through repeated joins, repeated correlated scalar
subqueries, or a mix of both.

## Why and when

PostgreSQL does not deduplicate repeated joins or correlated scalar subqueries
against the same table on the same key. Each extra join or scalar subquery adds
another index probe per outer row and duplicates I/O and CPU work.

Enable this rule on query-heavy services and reporting SQL where queries evolve
over time by tacking on extra `LEFT JOIN` clauses or inline scalar subqueries
for additional columns from an already-fetched relation.

## What it catches

Within a single `SELECT`, `UPDATE`, `DELETE`, or CTE query, this rule flags a
second or subsequent keyed lookup of the same target relation when the
normalized lookup key matches an earlier lookup in the same scope:

- Repeated correlated scalar subqueries on the same relation and key, including
  unqualified inner key columns (`WHERE id = o.account_id`) and differing table
  aliases (`accounts a1` vs `accounts a2`).
- Repeated `JOIN` clauses (`INNER`, `LEFT`, `RIGHT`, `FULL`, `USING`) against
  the same relation on the same key, regardless of equality operand order
  (`a.id = o.account_id` vs `o.account_id = a.id`).
- A `JOIN` combined with a correlated scalar subquery against the same relation
  and key.
- Equivalent PostgreSQL cast spellings (`o.raw_id::uuid` and
  `CAST(o.raw_id AS uuid)`).

The rule stays silent when the lookups are not proven to read the same row:

- Different outer key expressions (`o.buyer_id` vs `o.seller_id`) or different
  casts (`o.raw_id::uuid` vs `o.raw_id::text`).
- Different additional filter predicates (such as `addr.kind = 'billing'` vs
  `addr.kind = 'shipping'`).
- A lookup paired with a correlated aggregate (`COUNT(*)`, `SUM(...)`, etc.) or
  an `EXISTS (...)` test.
- Mutually exclusive `CASE WHEN ... THEN (SELECT ...) ELSE (SELECT ...) END`
  branches.
- Volatile key expressions (`gen_random_uuid()`, `random()`, `clock_timestamp()`,
  sequence-advancing functions), or ambiguous/unprovable expressions.

Findings point at the duplicate join or scalar subquery and name the line of the
first lookup.

## Options and defaults

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
  queries, or `executorNames: []` to scan only `.sql` files selected by
  `sqlInclude`. Setting both `executorNames` and `extraExecutorNames` is a
  configuration error. See
  [executor selection](../configuration/rules.md#postgresql-executor-selection).
- `unanalyzable`: shared uncertainty policy (`report`, `advise`, or `ignore`;
  default `report`). `report` fails on dynamically constructed or unparseable
  SQL, `advise` emits a non-failing advisory, and `ignore` omits residual
  diagnostics. Legacy `unanalyzableSql` (`fail`, `advise`, `ignore`) is accepted
  as an alias when `unanalyzable` is not set.

A `projects: [<name>]` entry inherits
[`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when
that block is configured.

## Valid example

Combine the lookups into a single join and project every needed column from it:

```sql
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
SELECT
  o.id,
  (SELECT display_name FROM accounts WHERE id = o.account_id) AS account_name,
  (SELECT tier FROM accounts WHERE id = o.account_id) AS account_tier
FROM orders o;
```

Two joins against the same `accounts` row on `o.account_id`:

```sql
SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = o.account_id
LEFT JOIN accounts a2 ON o.account_id = a2.id;
```

## Fix

Replace the repeated joins or correlated scalar subqueries with one `LEFT JOIN`
(or `INNER JOIN` when the target row is guaranteed to exist) and select all
needed columns from that single binding.

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
