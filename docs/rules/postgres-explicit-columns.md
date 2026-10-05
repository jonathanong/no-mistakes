# `postgres-explicit-columns`

Require an explicit column list instead of `SELECT *` or `RETURNING *` when the
relation is wider than `maxColumns`, or when it is named in `relations`.

```yaml
rules:
  - rule: postgres-explicit-columns
    options:
      include: ["src/**/*.ts"]
      exclude: ["**/*.test.ts"]
      sqlInclude: []
      schemaCatalogPath: db/schema.json
      maxColumns: 12
      relations: [accounts]
      allowWholeRowFunctions: [row_to_json, to_json, to_jsonb, json_agg, jsonb_agg]
      checkReturning: true
      unanalyzableSql: ignore
```

## Why and when

`SELECT *` fetches every column, including large values the caller
never reads, and it blocks index-only scans. A column added later is also
returned to every star caller, which is how sensitive columns leak into API
responses. Enable this on application queries that should name the columns they
use. Views stay allowed unless you list them in `relations`, because a view's
column list is itself a reviewed contract.

## What it catches/requires

A standalone `TABLE accounts` reads every column of `accounts`, so it is
checked like `SELECT * FROM accounts` when that relation is configured.
The same check applies when `TABLE accounts` is an arm of a supported set
operation, such as `SELECT id FROM safe UNION ALL TABLE accounts`. This
also covers TABLE arms inside UPDATE and DELETE subqueries; an EXISTS arm
remains exempt from explicit-column stars.

The rule reads star projections from the statement pass and column counts from
the schema catalog. It reports a star when the catalog table has more than
`maxColumns` columns, or when the relation is listed in `relations`.
`maxColumns: 0` reports every catalog table, whatever its width.

It does not report `EXISTS (SELECT *)`, `NOT EXISTS (SELECT *)`, `COUNT(*)`, a
star inside `allowWholeRowFunctions`, or a star over a CTE or derived-table
alias. Recursive WITH makes every sibling alias local in each body; non-recursive
WITH makes an alias local only after its definition. Schema-qualified physical
relations remain checked. The inner `SELECT *` of that CTE or subquery is still checked. One
finding is emitted per star and relation. `RETURNING *` uses the same rule when
`checkReturning` is true, including data-modifying CTEs. Bare RETURNING stars
expand the mutation target; qualified stars can also refer to UPDATE FROM or
DELETE USING tables. Schema-qualified relations retain their full catalog
identity, and table-valued functions are not catalog tables.

With no `relations` entries the rule still reports catalog tables wider than the
default `maxColumns` of 12. `schemaCatalogPath` is required, so enabling the
rule with no options is a config error rather than a silent skip.

## Options and defaults

`include`, `exclude`, `importSpecifier`, `executorNames`, and `unanalyzableSql`
match `postgres-required-predicates`. `unanalyzableSql` defaults to `fail`.

`sqlInclude` defaults to `[]`, so `.sql` files are not scanned until you set
globs. `schemaCatalogPath` has no default, is required, and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `maxColumns` defaults
to `12`; `0` means every catalog table. Values outside `0` through `4294967295` are configuration errors.
`relations` defaults to `[]`. An empty name or a duplicate name is a config
error. `allowWholeRowFunctions` defaults to `row_to_json`, `to_json`,
`to_jsonb`, `json_agg`, and `jsonb_agg` (matched case-insensitively). An empty
list means no function is exempt. An empty or duplicate name is a config error.
`checkReturning` defaults to `true`.

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

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

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
`postgres-explicit-columns`. Findings point to each star projection, and
embedded SQL parse failures point to their host call or declaration, so line
suppression applies at the affected query.

## Related rules

`postgres-required-predicates` requires predicates on named columns.
`postgres-sql-shape-policy` bans selected query shapes. This rule only cares
about star projections and `RETURNING *`.
