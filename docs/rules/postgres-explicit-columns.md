# `postgres-explicit-columns`

Require an explicit column list instead of `SELECT *` or `RETURNING *` when the
relation is wider than `maxColumns`, or when it is named in `relations`.

```yaml
rules:
  - rule: postgres-explicit-columns
    options:
      include: ['src/**/*.ts']
      exclude: ['**/*.test.ts']
      sqlInclude: []
      schemaCatalogPath: db/schema.json
      maxColumns: 12
      relations: [accounts]
      allowWholeRowFunctions: [row_to_json, to_json, to_jsonb, json_agg, jsonb_agg]
      checkReturning: true
      unanalyzableSql: ignore
```

## Why and when

`SELECT *` fetches and detoasts every column, including large values the caller
never reads, and it blocks index-only scans. A column added later is also
returned to every star caller, which is how sensitive columns leak into API
responses. Enable this on application queries that should name the columns they
use. Views stay allowed unless you list them in `relations`, because a view's
column list is itself a reviewed contract.

## What it catches/requires

The rule reads star projections from the statement pass and column counts from
the schema catalog. It reports a star when the catalog table has more than
`maxColumns` columns, or when the relation is listed in `relations`.
`maxColumns: 0` reports every catalog table, whatever its width.

It does not report `EXISTS (SELECT *)`, `NOT EXISTS (SELECT *)`, `COUNT(*)`, a
star inside `allowWholeRowFunctions`, or a star over a CTE or derived-table
alias. The inner `SELECT *` of that CTE or subquery is still checked. One
finding is emitted per star and relation. `RETURNING *` uses the same rule when
`checkReturning` is true.

With no `relations` entries the rule still reports catalog tables wider than the
default `maxColumns` of 12. `schemaCatalogPath` is required, so enabling the
rule with no options is a config error rather than a silent skip.

## Options and defaults

`include`, `exclude`, `importSpecifier`, `executorNames`, and `unanalyzableSql`
match `postgres-required-predicates`. `unanalyzableSql` defaults to `fail`.

`sqlInclude` defaults to `[]`, so `.sql` files are not scanned until you set
globs. `schemaCatalogPath` has no default and is required. `maxColumns` defaults
to `12`; `0` means every catalog table, and a negative value is a config error.
`relations` defaults to `[]`. An empty name or a duplicate name is a config
error. `allowWholeRowFunctions` defaults to `row_to_json`, `to_json`,
`to_jsonb`, `json_agg`, and `jsonb_agg` (matched case-insensitively). An empty
list means no function is exempt. An empty or duplicate name is a config error.
`checkReturning` defaults to `true`.

## Valid example

```sql
SELECT id, status FROM orders WHERE id = $1;
SELECT * FROM tags;
SELECT EXISTS (SELECT * FROM orders WHERE account_id = $1);
SELECT COUNT(*) FROM orders;
SELECT row_to_json(o.*) FROM orders o WHERE id = $1;
WITH r AS (SELECT id, status FROM orders) SELECT * FROM r;
```

`tags` has three columns, under the default limit of 12. `orders` is wide, so
the explicit list and the `row_to_json` wrapper are the shapes that pass.

## Counterexample

```sql
SELECT * FROM orders WHERE id = $1;
SELECT o.*, a.name FROM orders o JOIN accounts a ON a.id = o.account_id;
UPDATE orders SET status = 'paid' WHERE id = $1 RETURNING *;
WITH recent AS (SELECT * FROM orders WHERE id > $1) SELECT id FROM recent;
```

`orders` has 40 columns. The join reports `orders`. It also reports `accounts`
when that name is in `relations` and the query stars it. The CTE reports the
inner `SELECT *`.

## Fix

List the columns the caller uses:

```sql
SELECT id, status FROM orders WHERE id = $1;
```

For a whole-row JSON payload, keep the star inside a function listed in
`allowWholeRowFunctions`, such as `row_to_json(o.*)`.

## Suppression

SQL and TypeScript honor `-- no-mistakes-disable-line`,
`-- no-mistakes-disable-next-line`, and `-- no-mistakes-disable-file` for
`postgres-explicit-columns`.

## Related rules

`postgres-required-predicates` requires predicates on named columns.
`postgres-sql-shape-policy` bans selected query shapes. This rule only cares
about star projections and `RETURNING *`.
